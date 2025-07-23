use mtu_checker::{check_privileges, core::MtuResult, core::MtuRunner, init_logging, ui::Config};
use std::process;
use tokio::signal;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    let config = Config::parse_args();

    if let Err(e) = init_logging(config.verbose, config.debug) {
        eprintln!("Failed to initialize logging: {}", e);
        process::exit(1);
    }

    if !check_privileges() {
        warn!("Running without root privileges. ICMP operations may fail on some systems.");
        if !config.is_json_output() {
            eprintln!("Warning: You may need to run this tool with elevated privileges (sudo) for ICMP ping operations.");
            eprintln!("On some systems, you might need to set capabilities: sudo setcap cap_net_raw+ep /path/to/mtu-checker");
            eprintln!();
        }
    }

    let result = run_with_shutdown(config).await;

    match result {
        Ok(()) => {
            info!("MTU checker completed successfully");
            process::exit(0);
        }
        Err(e) => {
            error!("MTU checker failed: {}", e);
            if !e.category().is_empty() {
                error!("Error category: {}", e.category());
            }

            eprintln!("Error: {}", e);

            match e.category() {
                "connectivity" => {
                    eprintln!("Suggestions:");
                    eprintln!("• Check your internet connection");
                    eprintln!("• Verify the target IP address is reachable");
                    eprintln!("• Try using --skip-connectivity-test flag");
                    eprintln!("• Check if firewall is blocking ICMP packets");
                }
                "permissions" => {
                    eprintln!("Suggestions:");
                    eprintln!("• Run with elevated privileges: sudo mtu-checker");
                    eprintln!("• Set capabilities: sudo setcap cap_net_raw+ep mtu-checker");
                    eprintln!("• Check if your system supports raw sockets");
                }
                "configuration" => {
                    eprintln!("Suggestions:");
                    eprintln!("• Check your command line arguments");
                    eprintln!("• Use --help to see available options");
                    eprintln!("• Ensure MTU range is valid (min < max)");
                }
                "timeout" => {
                    eprintln!("Suggestions:");
                    eprintln!("• Increase timeout with --timeout-ms");
                    eprintln!("• Check network stability");
                    eprintln!("• Try a different target IP address");
                }
                _ => {
                    eprintln!("Use --verbose or --debug for more information");
                }
            }

            process::exit(1);
        }
    }
}

async fn run_with_shutdown(config: Config) -> MtuResult<()> {
    let runner = MtuRunner::new(config)?;

    runner.print_runtime_estimate();

    let shutdown_future = setup_shutdown_handler();
    let main_task = runner.run();

    tokio::select! {
        result = main_task => {
            result
        }
        _ = shutdown_future => {
            info!("Received shutdown signal, stopping MTU checker");
            println!("\nShutdown signal received. Stopping gracefully...");
            Ok(())
        }
    }
}

async fn setup_shutdown_handler() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }
}
