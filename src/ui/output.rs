use crate::network::ping::PingStats;
use crate::ui::config::Config;
use colored::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutput {
    pub timestamp: String,
    pub config: JsonConfig,
    pub results: Vec<JsonMtuResult>,
    pub optimal_mtu: Option<JsonMtuResult>,
    pub summary: JsonSummary,
    pub execution_time: JsonDuration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonConfig {
    pub target: String,
    pub min_mtu: u16,
    pub max_mtu: u16,
    pub step: u16,
    pub requests_per_mtu: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonMtuResult {
    pub mtu: u16,
    pub total_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub packet_loss_percent: f64,
    pub min_latency_ms: Option<f64>,
    pub max_latency_ms: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub success_rate: f64,
    pub test_duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonSummary {
    pub total_mtu_tests: usize,
    pub successful_mtu_tests: usize,
    pub total_ping_requests: u32,
    pub total_successful_pings: u32,
    pub overall_success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonDuration {
    pub total_seconds: f64,
    pub formatted: String,
}

impl From<&PingStats> for JsonMtuResult {
    fn from(stats: &PingStats) -> Self {
        Self {
            mtu: stats.mtu,
            total_requests: stats.total_requests,
            successful_requests: stats.successful_requests,
            failed_requests: stats.failed_requests,
            packet_loss_percent: stats.packet_loss_percent,
            min_latency_ms: stats.min_latency.map(|d| d.as_secs_f64() * 1000.0),
            max_latency_ms: stats.max_latency.map(|d| d.as_secs_f64() * 1000.0),
            avg_latency_ms: stats.avg_latency.map(|d| d.as_secs_f64() * 1000.0),
            success_rate: stats.success_rate(),
            test_duration_ms: stats.total_time.as_millis() as u64,
        }
    }
}

pub struct OutputManager {
    config: Config,
    start_time: Instant,
}

impl OutputManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            start_time: Instant::now(),
        }
    }

    pub fn print_header(&self) {
        if self.config.is_json_output() {
            return;
        }

        println!(
            "{}",
            "═══════════════════════════════════════════════════════════"
                .blue()
                .bold()
        );
        println!(
            "{}",
            "                    MTU CHECKER v0.1.0                    "
                .blue()
                .bold()
        );
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════"
                .blue()
                .bold()
        );
        println!();

        println!("{} {}", "Target:".yellow().bold(), self.config.target);
        println!(
            "{} {} - {}",
            "MTU Range:".yellow().bold(),
            self.config.min_mtu,
            self.config.max_mtu
        );
        println!("{} {}", "Step Size:".yellow().bold(), self.config.step);
        println!(
            "{} {}",
            "Requests per MTU:".yellow().bold(),
            self.config.requests
        );
        println!(
            "{} {}ms",
            "Timeout:".yellow().bold(),
            self.config.timeout_ms
        );

        if let Some(ref interface) = self.config.interface {
            println!("{} {}", "Interface:".yellow().bold(), interface);
        }

        println!();
    }

    pub fn print_connectivity_test(&self, success: bool) {
        if self.config.is_json_output() {
            return;
        }

        print!("{} ", "Testing connectivity...".cyan());
        io::stdout().flush().unwrap();

        if success {
            println!("{}", "✓ Connected".green().bold());
        } else {
            println!("{}", "✗ Failed".red().bold());
        }
        println!();
    }

    pub fn print_test_start(&self, total_tests: usize) {
        if self.config.is_json_output() {
            return;
        }

        println!(
            "{} {} MTU sizes",
            "Starting tests for".cyan(),
            total_tests.to_string().yellow().bold()
        );
        println!("{}", "─".repeat(80).bright_black());
        println!();
    }

    pub fn print_mtu_test_start(&self, mtu: u16, current: usize, total: usize) {
        if self.config.is_json_output() {
            return;
        }

        print!(
            "[{}/{}] Testing MTU {}: ",
            current.to_string().cyan(),
            total.to_string().cyan(),
            mtu.to_string().yellow().bold()
        );
        io::stdout().flush().unwrap();
    }

    pub fn print_ping_progress(&self, current: u32, total: u32, success: bool) {
        if self.config.is_json_output() {
            return;
        }

        let symbol = if success { "●".green() } else { "●".red() };
        print!("{}", symbol);

        if current == total {
            print!(" ");
        }

        io::stdout().flush().unwrap();
    }

    pub fn print_mtu_result(&self, stats: &PingStats) {
        if self.config.is_json_output() {
            return;
        }

        let status = if stats.is_successful() {
            format!(
                "✓ {:.1}ms avg",
                stats.avg_latency.unwrap_or(Duration::ZERO).as_secs_f64() * 1000.0
            )
            .green()
        } else {
            "✗ Failed".red()
        };

        let loss_info = if stats.packet_loss_percent > 0.0 {
            format!(" ({:.1}% loss)", stats.packet_loss_percent).yellow()
        } else {
            "".normal()
        };

        println!("{}{}", status, loss_info);

        if self.config.verbose && stats.is_successful() {
            let min_ms = stats.min_latency.unwrap_or(Duration::ZERO).as_secs_f64() * 1000.0;
            let max_ms = stats.max_latency.unwrap_or(Duration::ZERO).as_secs_f64() * 1000.0;
            let avg_ms = stats.avg_latency.unwrap_or(Duration::ZERO).as_secs_f64() * 1000.0;

            println!(
                "    {} {:.1}ms | {} {:.1}ms | {} {:.1}ms",
                "min:".bright_black(),
                min_ms,
                "max:".bright_black(),
                max_ms,
                "avg:".bright_black(),
                avg_ms
            );
        }
    }

    pub fn print_final_results(&self, results: &[PingStats], optimal: Option<&PingStats>) {
        if self.config.is_json_output() {
            self.print_json_results(results, optimal);
            return;
        }

        let execution_time = self.start_time.elapsed();

        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════"
                .blue()
                .bold()
        );
        println!(
            "{}",
            "                        RESULTS                           "
                .blue()
                .bold()
        );
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════"
                .blue()
                .bold()
        );
        println!();

        if let Some(best) = optimal {
            println!("{}", "🏆 OPTIMAL MTU FOUND".green().bold());
            println!();
            println!(
                "{} {}",
                "Best MTU Size:".yellow().bold(),
                best.mtu.to_string().green().bold()
            );

            if let Some(avg_latency) = best.avg_latency {
                println!(
                    "{} {:.2}ms",
                    "Average Latency:".yellow().bold(),
                    (avg_latency.as_secs_f64() * 1000.0)
                        .to_string()
                        .green()
                        .bold()
                );
            }

            println!(
                "{} {:.1}%",
                "Success Rate:".yellow().bold(),
                best.success_rate().to_string().green().bold()
            );

            if best.packet_loss_percent > 0.0 {
                println!(
                    "{} {:.1}%",
                    "Packet Loss:".yellow().bold(),
                    best.packet_loss_percent.to_string().yellow()
                );
            }

            if let Some(min_lat) = best.min_latency {
                if let Some(max_lat) = best.max_latency {
                    println!(
                        "{} {:.2}ms - {:.2}ms",
                        "Latency Range:".yellow().bold(),
                        min_lat.as_secs_f64() * 1000.0,
                        max_lat.as_secs_f64() * 1000.0
                    );
                }
            }
        } else {
            println!("{}", "❌ NO OPTIMAL MTU FOUND".red().bold());
            println!();
            println!("All ping attempts failed. This could indicate:");
            println!("• Network connectivity issues");
            println!("• Firewall blocking ICMP packets");
            println!("• MTU range too restrictive");
            println!("• Target host unreachable");
        }

        println!();
        self.print_summary_table(results);

        println!();
        println!(
            "{} {}",
            "Total Execution Time:".yellow().bold(),
            format_duration(execution_time).cyan()
        );
        println!();
    }

    fn print_summary_table(&self, results: &[PingStats]) {
        if results.is_empty() {
            return;
        }

        println!("{}", "DETAILED RESULTS".yellow().bold());
        println!("{}", "─".repeat(80).bright_black());

        println!(
            "{:<6} {:<12} {:<10} {:<12} {:<10} {:<12}",
            "MTU".bold(),
            "Avg Latency".bold(),
            "Min/Max".bold(),
            "Success".bold(),
            "Loss %".bold(),
            "Status".bold()
        );
        println!("{}", "─".repeat(80).bright_black());

        for stats in results {
            let mtu_str = stats.mtu.to_string();

            let (avg_str, minmax_str, success_str, loss_str, status_str) = if stats.is_successful()
            {
                let avg_ms = stats.avg_latency.unwrap_or(Duration::ZERO).as_secs_f64() * 1000.0;
                let min_ms = stats.min_latency.unwrap_or(Duration::ZERO).as_secs_f64() * 1000.0;
                let max_ms = stats.max_latency.unwrap_or(Duration::ZERO).as_secs_f64() * 1000.0;

                (
                    format!("{:.2}ms", avg_ms).green().to_string(),
                    format!("{:.1}/{:.1}", min_ms, max_ms).normal().to_string(),
                    format!("{}/{}", stats.successful_requests, stats.total_requests)
                        .green()
                        .to_string(),
                    if stats.packet_loss_percent > 0.0 {
                        format!("{:.1}%", stats.packet_loss_percent)
                            .yellow()
                            .to_string()
                    } else {
                        "0%".green().to_string()
                    },
                    "✓ OK".green().to_string(),
                )
            } else {
                (
                    "---".red().to_string(),
                    "---".red().to_string(),
                    format!("0/{}", stats.total_requests).red().to_string(),
                    "100%".red().to_string(),
                    "✗ FAIL".red().to_string(),
                )
            };

            println!(
                "{:<6} {:<12} {:<10} {:<12} {:<10} {:<12}",
                mtu_str, avg_str, minmax_str, success_str, loss_str, status_str
            );
        }
    }

    fn print_json_results(&self, results: &[PingStats], optimal: Option<&PingStats>) {
        let execution_time = self.start_time.elapsed();

        let json_results: Vec<JsonMtuResult> = results.iter().map(JsonMtuResult::from).collect();

        let total_requests: u32 = results.iter().map(|r| r.total_requests).sum();
        let total_successful: u32 = results.iter().map(|r| r.successful_requests).sum();
        let successful_tests = results.iter().filter(|r| r.is_successful()).count();

        let summary = JsonSummary {
            total_mtu_tests: results.len(),
            successful_mtu_tests: successful_tests,
            total_ping_requests: total_requests,
            total_successful_pings: total_successful,
            overall_success_rate: if total_requests > 0 {
                (total_successful as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            },
        };

        let output = JsonOutput {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
            config: JsonConfig {
                target: self.config.target.to_string(),
                min_mtu: self.config.min_mtu,
                max_mtu: self.config.max_mtu,
                step: self.config.step,
                requests_per_mtu: self.config.requests,
                timeout_ms: self.config.timeout_ms,
            },
            results: json_results,
            optimal_mtu: optimal.map(JsonMtuResult::from),
            summary,
            execution_time: JsonDuration {
                total_seconds: execution_time.as_secs_f64(),
                formatted: format_duration(execution_time),
            },
        };

        match serde_json::to_string_pretty(&output) {
            Ok(json_str) => println!("{}", json_str),
            Err(e) => eprintln!("Error serializing results to JSON: {}", e),
        }
    }

    pub fn print_error(&self, error: &str) {
        if self.config.is_json_output() {
            let error_output = serde_json::json!({
                "error": error,
                "timestamp": SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&error_output).unwrap_or_default()
            );
        } else {
            eprintln!("{} {}", "Error:".red().bold(), error);
        }
    }

    pub fn print_warning(&self, warning: &str) {
        if self.config.is_json_output() {
            return;
        }
        eprintln!("{} {}", "Warning:".yellow().bold(), warning);
    }

    pub fn print_debug(&self, message: &str) {
        if !self.config.debug || self.config.is_json_output() {
            return;
        }
        eprintln!("{} {}", "Debug:".blue().bold(), message.bright_black());
    }

    pub fn print_verbose(&self, message: &str) {
        if !self.config.verbose || self.config.is_json_output() {
            return;
        }
        println!("{}", message.bright_black());
    }
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    let millis = duration.subsec_millis();

    if minutes > 0 {
        format!("{}m {}.{:03}s", minutes, seconds, millis)
    } else if seconds > 0 {
        format!("{}.{:03}s", seconds, millis)
    } else {
        format!("{}ms", duration.as_millis())
    }
}
