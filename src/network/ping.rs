use crate::core::error::{MtuError, MtuResult};
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, trace, warn};

#[derive(Debug, Clone)]
pub struct PingResult {
    pub target: IpAddr,
    pub mtu: u16,
    pub sequence: u16,
    pub latency: Duration,
    pub success: bool,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct PingStats {
    pub target: IpAddr,
    pub mtu: u16,
    pub total_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub min_latency: Option<Duration>,
    pub max_latency: Option<Duration>,
    pub avg_latency: Option<Duration>,
    pub packet_loss_percent: f64,
    pub total_time: Duration,
    pub results: Vec<PingResult>,
}

impl PingStats {
    pub fn new(target: IpAddr, mtu: u16) -> Self {
        Self {
            target,
            mtu,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            min_latency: None,
            max_latency: None,
            avg_latency: None,
            packet_loss_percent: 0.0,
            total_time: Duration::ZERO,
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: PingResult) {
        self.results.push(result.clone());
        self.total_requests += 1;

        if result.success {
            self.successful_requests += 1;

            match self.min_latency {
                None => self.min_latency = Some(result.latency),
                Some(min) if result.latency < min => self.min_latency = Some(result.latency),
                _ => {}
            }

            match self.max_latency {
                None => self.max_latency = Some(result.latency),
                Some(max) if result.latency > max => self.max_latency = Some(result.latency),
                _ => {}
            }

            let successful_count = self.successful_requests as u64;
            let total_latency: u64 = self
                .results
                .iter()
                .filter(|r| r.success)
                .map(|r| r.latency.as_micros() as u64)
                .sum();

            if successful_count > 0 {
                self.avg_latency = Some(Duration::from_micros(total_latency / successful_count));
            }
        } else {
            self.failed_requests += 1;
        }

        self.packet_loss_percent =
            (self.failed_requests as f64 / self.total_requests as f64) * 100.0;
    }

    pub fn is_successful(&self) -> bool {
        self.successful_requests > 0
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
}

pub struct PingManager {
    timeout_duration: Duration,
}

impl PingManager {
    pub fn new(timeout_duration: Duration, interface: Option<String>) -> MtuResult<Self> {
        if let Some(iface) = interface {
            debug!(
                "Interface binding requested: {}, but may not be supported",
                iface
            );
        }

        Ok(Self { timeout_duration })
    }

    pub async fn ping_once(
        &self,
        target: IpAddr,
        mtu: u16,
        sequence: u16,
    ) -> MtuResult<PingResult> {
        trace!(
            "Starting ping to {} with MTU {} (seq: {})",
            target,
            mtu,
            sequence
        );

        let start_time = Instant::now();

        let payload_size = if mtu > 28 {
            std::cmp::min((mtu - 28) as usize, 65507)
        } else {
            32
        };

        let mut payload = vec![0x42u8; payload_size];

        if payload.len() >= 2 {
            payload[0] = (sequence >> 8) as u8;
            payload[1] = (sequence & 0xFF) as u8;
        }

        let ping_future = async {
            match surge_ping::ping(target, &payload).await {
                Ok((packet, duration)) => Ok((packet, duration)),
                Err(e) => Err(e),
            }
        };

        let result = timeout(self.timeout_duration, ping_future).await;

        match result {
            Ok(Ok((_packet, duration))) => {
                let ping_result = PingResult {
                    target,
                    mtu,
                    sequence,
                    latency: duration,
                    success: true,
                    timestamp: start_time,
                };
                trace!("Ping successful: {:?}", ping_result);
                Ok(ping_result)
            }
            Ok(Err(e)) => {
                warn!("Ping failed to {} with MTU {}: {}", target, mtu, e);
                Ok(PingResult {
                    target,
                    mtu,
                    sequence,
                    latency: Duration::ZERO,
                    success: false,
                    timestamp: start_time,
                })
            }
            Err(_) => {
                warn!(
                    "Ping timeout to {} with MTU {} after {:?}",
                    target, mtu, self.timeout_duration
                );
                Ok(PingResult {
                    target,
                    mtu,
                    sequence,
                    latency: Duration::ZERO,
                    success: false,
                    timestamp: start_time,
                })
            }
        }
    }

    pub async fn ping_multiple(
        &self,
        target: IpAddr,
        mtu: u16,
        count: u32,
        delay_between_pings: Duration,
    ) -> MtuResult<PingStats> {
        debug!("Starting {} pings to {} with MTU {}", count, target, mtu);

        let start_time = Instant::now();
        let mut stats = PingStats::new(target, mtu);

        for i in 0..count {
            let result = self.ping_once(target, mtu, i as u16).await?;
            stats.add_result(result);

            if i < count - 1 && delay_between_pings > Duration::ZERO {
                tokio::time::sleep(delay_between_pings).await;
            }
        }

        stats.total_time = start_time.elapsed();

        debug!(
            "Completed pings for MTU {}: {}/{} successful, {:.1}% packet loss, avg latency: {:?}",
            mtu,
            stats.successful_requests,
            stats.total_requests,
            stats.packet_loss_percent,
            stats.avg_latency
        );

        Ok(stats)
    }

    pub async fn test_connectivity(&self, target: IpAddr) -> MtuResult<bool> {
        debug!("Testing connectivity to {}", target);

        for attempt in 0..3 {
            let result = self.ping_once(target, 576, attempt).await?;
            if result.success {
                debug!(
                    "Connectivity test successful to {} on attempt {}",
                    target,
                    attempt + 1
                );
                return Ok(true);
            }

            if attempt < 2 {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        warn!("All connectivity test attempts failed to {}", target);
        Err(MtuError::NetworkConnectivity { target })
    }
}

pub fn calculate_optimal_mtu(stats_list: &[PingStats]) -> Option<&PingStats> {
    stats_list
        .iter()
        .filter(|stats| stats.is_successful())
        .min_by(|a, b| match (a.avg_latency, b.avg_latency) {
            (Some(a_lat), Some(b_lat)) => {
                let latency_cmp = a_lat.cmp(&b_lat);
                if latency_cmp == std::cmp::Ordering::Equal {
                    b.mtu.cmp(&a.mtu)
                } else {
                    latency_cmp
                }
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                let a_success = a.success_rate();
                let b_success = b.success_rate();
                if (a_success - b_success).abs() < f64::EPSILON {
                    b.mtu.cmp(&a.mtu)
                } else {
                    b_success
                        .partial_cmp(&a_success)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            }
        })
}
