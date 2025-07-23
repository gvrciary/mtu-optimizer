#![warn(clippy::all)]

pub mod core;
pub mod network;
pub mod ui;

pub use core::{MtuError, MtuResult, MtuRunner};
pub use network::{calculate_optimal_mtu, PingManager, PingResult, PingStats};
pub use ui::{Config, OutputFormat, OutputManager};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_TARGET: &str = "8.8.8.8";
pub const MIN_MTU: u16 = 576;
pub const MAX_MTU: u16 = 9000;
pub const STANDARD_MTU: u16 = 1500;

pub fn init_logging(verbose: bool, debug: bool) -> MtuResult<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = if debug {
        EnvFilter::new("mtu_checker=debug")
    } else if verbose {
        EnvFilter::new("mtu_checker=info")
    } else {
        EnvFilter::new("mtu_checker=warn")
    };

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .init();

    Ok(())
}

pub fn check_privileges() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }

    #[cfg(windows)]
    {
        true
    }
}

pub fn get_recommended_mtu_range(network_type: &str) -> (u16, u16) {
    match network_type.to_lowercase().as_str() {
        "ethernet" => (1200, 1500),
        "wifi" => (1200, 1500),
        "mobile" | "cellular" => (1200, 1400),
        "vpn" => (1200, 1450),
        "dsl" => (1200, 1492),
        "pppoe" => (1200, 1492),
        "jumbo" => (1500, 9000),
        _ => (1200, 1500),
    }
}
