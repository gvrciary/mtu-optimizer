use clap::Parser;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(short, long, default_value = "8.8.8.8")]
    pub target: IpAddr,

    #[arg(long, default_value = "1200")]
    pub min_mtu: u16,

    #[arg(long, default_value = "1500")]
    pub max_mtu: u16,

    #[arg(long, default_value = "8")]
    pub step: u16,

    #[arg(short, long, default_value = "5")]
    pub requests: u32,

    #[arg(short = 'T', long, default_value = "3000")]
    pub timeout_ms: u64,

    #[arg(short, long)]
    pub interface: Option<String>,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub debug: bool,

    #[arg(short, long, default_value = "text")]
    pub format: OutputFormat,
    
    #[arg(long)]
    pub skip_connectivity_test: bool,

    #[arg(long, default_value = "10")]
    pub max_concurrent: usize,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Text,
    Json,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" | "txt" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!(
                "Invalid output format: {}. Valid options: text, json",
                s
            )),
        }
    }
}

impl Config {
    pub fn parse_args() -> Self {
        Config::parse()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.min_mtu >= self.max_mtu {
            return Err(format!(
                "Minimum MTU ({}) must be less than maximum MTU ({})",
                self.min_mtu, self.max_mtu
            ));
        }

        if self.min_mtu < 576 {
            return Err("Minimum MTU cannot be less than 576 bytes (IPv4 minimum)".to_string());
        }

        if self.max_mtu > 9000 {
            return Err("Maximum MTU cannot exceed 9000 bytes".to_string());
        }

        if self.step == 0 {
            return Err("Step size must be greater than 0".to_string());
        }

        if self.requests == 0 {
            return Err("Number of requests must be greater than 0".to_string());
        }

        if self.requests > 100 {
            return Err("Number of requests per MTU cannot exceed 100".to_string());
        }

        if self.timeout_ms < 100 {
            return Err("Timeout must be at least 100ms".to_string());
        }

        if self.timeout_ms > 30000 {
            return Err("Timeout cannot exceed 30 seconds".to_string());
        }

        if self.max_concurrent == 0 {
            return Err("Maximum concurrent operations must be greater than 0".to_string());
        }

        Ok(())
    }

    pub fn mtu_range(&self) -> impl Iterator<Item = u16> {
        (self.min_mtu..=self.max_mtu).step_by(self.step as usize)
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_ms)
    }

    pub fn is_json_output(&self) -> bool {
        matches!(self.format, OutputFormat::Json)
    }
}
