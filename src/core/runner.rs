use crate::core::error::{MtuError, MtuResult};
use crate::network::ping::{calculate_optimal_mtu, PingManager, PingStats};
use crate::ui::config::Config;
use crate::ui::output::OutputManager;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

pub struct MtuRunner {
    config: Config,
    ping_manager: PingManager,
    output_manager: OutputManager,
}

impl MtuRunner {
    pub fn new(config: Config) -> MtuResult<Self> {
        config.validate().map_err(|e| MtuError::config(e))?;

        let ping_manager = PingManager::new(config.timeout(), config.interface.clone())?;
        let output_manager = OutputManager::new(config.clone());

        Ok(Self {
            config,
            ping_manager,
            output_manager,
        })
    }

    pub async fn run(&self) -> MtuResult<()> {
        info!("Starting MTU discovery process");

        self.output_manager.print_header();

        if !self.config.skip_connectivity_test {
            if let Err(e) = self.test_connectivity().await {
                self.output_manager.print_error(&format!(
                    "Connectivity test failed: {}. Use --skip-connectivity-test to bypass this check.",
                    e
                ));
                return Err(e);
            }
        }

        let mtu_values: Vec<u16> = self.config.mtu_range().collect();

        if mtu_values.is_empty() {
            return Err(MtuError::config(
                "No MTU values to test in the specified range",
            ));
        }

        self.output_manager.print_test_start(mtu_values.len());

        let results = self.discover_optimal_mtu(&mtu_values).await?;
        let optimal_mtu = calculate_optimal_mtu(&results);

        self.output_manager
            .print_final_results(&results, optimal_mtu);

        if optimal_mtu.is_some() {
            info!("MTU discovery completed successfully");
            Ok(())
        } else {
            warn!("MTU discovery completed but no optimal MTU found");
            Err(MtuError::NoValidResults)
        }
    }

    async fn test_connectivity(&self) -> MtuResult<()> {
        debug!("Testing connectivity to {}", self.config.target);

        let success = self
            .ping_manager
            .test_connectivity(self.config.target)
            .await;

        match success {
            Ok(true) => {
                self.output_manager.print_connectivity_test(true);
                Ok(())
            }
            Ok(false) | Err(_) => {
                self.output_manager.print_connectivity_test(false);
                Err(MtuError::NetworkConnectivity {
                    target: self.config.target,
                })
            }
        }
    }

    async fn discover_optimal_mtu(&self, mtu_values: &[u16]) -> MtuResult<Vec<PingStats>> {
        let total_tests = mtu_values.len();
        let mut results = Vec::with_capacity(total_tests);

        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent));

        let progress_bar = if !self.config.is_json_output() && !self.config.verbose {
            let pb = ProgressBar::new(total_tests as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}/{len:3} MTU {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
                    .progress_chars("█▉▊▋▌▍▎▏ "),
            );
            Some(pb)
        } else {
            None
        };

        for (index, &mtu) in mtu_values.iter().enumerate() {
            let current_test = index + 1;

            if let Some(ref pb) = progress_bar {
                pb.set_message(format!("{}", mtu));
                pb.set_position(index as u64);
            } else {
                self.output_manager
                    .print_mtu_test_start(mtu, current_test, total_tests);
            }

            let _permit = semaphore
                .acquire()
                .await
                .map_err(|e| MtuError::internal(format!("Failed to acquire semaphore: {}", e)))?;

            let stats_result = self.test_mtu_size(mtu).await;

            match stats_result {
                Ok(stats) => {
                    if progress_bar.is_none() {
                        self.output_manager.print_mtu_result(&stats);
                    }
                    results.push(stats);
                }
                Err(e) => {
                    warn!("Failed to test MTU {}: {}", mtu, e);

                    if progress_bar.is_none() {
                        self.output_manager
                            .print_error(&format!("MTU {} test failed: {}", mtu, e));
                    }

                    let mut failed_stats = PingStats::new(self.config.target, mtu);
                    failed_stats.total_requests = self.config.requests;
                    failed_stats.failed_requests = self.config.requests;
                    failed_stats.packet_loss_percent = 100.0;
                    results.push(failed_stats);
                }
            }

            if index < mtu_values.len() - 1 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        if let Some(pb) = progress_bar {
            pb.finish_with_message("Complete");
            println!();
        }

        debug!("Completed testing {} MTU sizes", results.len());
        Ok(results)
    }

    async fn test_mtu_size(&self, mtu: u16) -> MtuResult<PingStats> {
        debug!("Testing MTU size: {}", mtu);

        if mtu < 576 || mtu > 9000 {
            return Err(MtuError::invalid_mtu(mtu, 576, 9000));
        }

        let start_time = Instant::now();

        let delay_between_pings = Duration::from_millis(200);
        let mut stats = self
            .ping_manager
            .ping_multiple(
                self.config.target,
                mtu,
                self.config.requests,
                delay_between_pings,
            )
            .await?;

        stats.total_time = start_time.elapsed();

        debug!(
            "MTU {} test completed: {}/{} successful, {:.1}% loss, avg latency: {:?}",
            mtu,
            stats.successful_requests,
            stats.total_requests,
            stats.packet_loss_percent,
            stats.avg_latency
        );

        Ok(stats)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn estimate_runtime(&self) -> Duration {
        let mtu_count = self.config.mtu_range().count();
        let pings_per_mtu = self.config.requests;
        let delay_between_pings = Duration::from_millis(200);
        let ping_timeout = self.config.timeout();

        let estimated_per_mtu = ping_timeout
            .saturating_mul(pings_per_mtu)
            .saturating_add(delay_between_pings.saturating_mul(pings_per_mtu))
            .saturating_add(Duration::from_millis(100));
        estimated_per_mtu.saturating_mul(mtu_count as u32)
    }

    pub fn print_runtime_estimate(&self) {
        if self.config.is_json_output() {
            return;
        }

        let estimated = self.estimate_runtime();
        let minutes = estimated.as_secs() / 60;
        let seconds = estimated.as_secs() % 60;

        if minutes > 0 {
            self.output_manager.print_verbose(&format!(
                "Estimated runtime: approximately {}m {}s",
                minutes, seconds
            ));
        } else {
            self.output_manager
                .print_verbose(&format!("Estimated runtime: approximately {}s", seconds));
        }
    }
}

pub async fn handle_shutdown_signal() -> MtuResult<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt())
            .map_err(|e| MtuError::internal(format!("Failed to register SIGINT handler: {}", e)))?;

        let mut sigterm = signal(SignalKind::terminate()).map_err(|e| {
            MtuError::internal(format!("Failed to register SIGTERM handler: {}", e))
        })?;

        tokio::select! {
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down gracefully");
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down gracefully");
            }
        }
    }

    #[cfg(windows)]
    {
        let mut ctrl_c = tokio::signal::ctrl_c()
            .map_err(|e| MtuError::internal(format!("Failed to register Ctrl+C handler: {}", e)))?;

        ctrl_c
            .await
            .map_err(|e| MtuError::internal(format!("Error waiting for Ctrl+C: {}", e)))?;

        info!("Received Ctrl+C, shutting down gracefully");
    }

    Ok(())
}
