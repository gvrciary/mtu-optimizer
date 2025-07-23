use std::net::IpAddr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MtuError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Network connectivity error: cannot reach {target}")]
    NetworkConnectivity { target: IpAddr },

    #[error("Ping error to {target} with MTU {mtu}: {source}")]
    Ping {
        target: IpAddr,
        mtu: u16,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Timeout error: operation took longer than {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Insufficient privileges: {message}")]
    InsufficientPrivileges { message: String },

    #[error("Invalid network interface: {interface}")]
    InvalidInterface { interface: String },

    #[error("Invalid MTU size: {mtu}. MTU must be between {min} and {max}")]
    InvalidMtu { mtu: u16, min: u16, max: u16 },

    #[error("Invalid IP address: {address}")]
    InvalidIpAddress { address: String },

    #[error("System I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("JSON serialization error: {source}")]
    JsonSerialization {
        #[from]
        source: serde_json::Error,
    },

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("No valid MTU results found. All ping attempts failed.")]
    NoValidResults,

    #[error("Operation cancelled by user")]
    Cancelled,
    
    #[error("Resource temporarily unavailable: {resource}")]
    ResourceUnavailable { resource: String },
}

impl MtuError {
    pub fn config<S: Into<String>>(message: S) -> Self {
        MtuError::Config {
            message: message.into(),
        }
    }

    pub fn ping<E>(target: IpAddr, mtu: u16, error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        MtuError::Ping {
            target,
            mtu,
            source: Box::new(error),
        }
    }

    pub fn timeout(timeout_ms: u64) -> Self {
        MtuError::Timeout { timeout_ms }
    }

    pub fn insufficient_privileges<S: Into<String>>(message: S) -> Self {
        MtuError::InsufficientPrivileges {
            message: message.into(),
        }
    }

    pub fn invalid_interface<S: Into<String>>(interface: S) -> Self {
        MtuError::InvalidInterface {
            interface: interface.into(),
        }
    }

    pub fn invalid_mtu(mtu: u16, min: u16, max: u16) -> Self {
        MtuError::InvalidMtu { mtu, min, max }
    }
    
    pub fn invalid_ip_address<S: Into<String>>(address: S) -> Self {
        MtuError::InvalidIpAddress {
            address: address.into(),
        }
    }
    
    pub fn internal<S: Into<String>>(message: S) -> Self {
        MtuError::Internal {
            message: message.into(),
        }
    }

    pub fn resource_unavailable<S: Into<String>>(resource: S) -> Self {
        MtuError::ResourceUnavailable {
            resource: resource.into(),
        }
    }
    
    pub fn is_recoverable(&self) -> bool {
        match self {
            MtuError::Timeout { .. } => true,
            MtuError::ResourceUnavailable { .. } => true,
            MtuError::Ping { .. } => true,
            MtuError::NetworkConnectivity { .. } => false,
            MtuError::Config { .. } => false,
            MtuError::InsufficientPrivileges { .. } => false,
            MtuError::InvalidInterface { .. } => false,
            MtuError::InvalidMtu { .. } => false,
            MtuError::InvalidIpAddress { .. } => false,
            MtuError::Io { .. } => false,
            MtuError::JsonSerialization { .. } => false,
            MtuError::Internal { .. } => false,
            MtuError::NoValidResults => false,
            MtuError::Cancelled => false,
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            MtuError::Config { .. } => "configuration",
            MtuError::NetworkConnectivity { .. } => "connectivity",
            MtuError::Ping { .. } => "ping",
            MtuError::Timeout { .. } => "timeout",
            MtuError::InsufficientPrivileges { .. } => "permissions",
            MtuError::InvalidInterface { .. } => "interface",
            MtuError::InvalidMtu { .. } => "mtu",
            MtuError::InvalidIpAddress { .. } => "address",
            MtuError::Io { .. } => "io",
            MtuError::JsonSerialization { .. } => "serialization",
            MtuError::Internal { .. } => "internal",
            MtuError::NoValidResults => "results",
            MtuError::Cancelled => "user",
            MtuError::ResourceUnavailable { .. } => "resource",
        }
    }
}

pub type MtuResult<T> = Result<T, MtuError>;

pub trait IntoMtuError<T> {
    fn into_mtu_error(self) -> MtuResult<T>;
}

impl<T, E> IntoMtuError<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn into_mtu_error(self) -> MtuResult<T> {
        self.map_err(|e| MtuError::internal(format!("External error: {}", e)))
    }
}
