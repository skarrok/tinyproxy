use std::net::SocketAddr;

use anyhow::anyhow;
use clap::Parser;
use clap::ValueEnum;
use serde::Serialize;
use serde_json::to_value;
use tracing::level_filters::LevelFilter;

/// A tiny and simple proxy with http, socks5 and tcp support
#[derive(Debug, Parser, Serialize)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Verbosity of logging
    #[arg(long, value_enum, env, default_value_t = LogLevel::Debug)]
    pub log_level: LogLevel,

    /// Format of logs
    #[arg(long, value_enum, env, default_value_t = LogFormat::Console)]
    pub log_format: LogFormat,

    // Proxy mode
    #[arg(value_enum, env)]
    pub proxy_mode: Mode,

    /// Listen address
    #[arg(long, short, env, default_value = "127.0.0.1:8000")]
    pub listen_address: SocketAddr,

    /// Remote address for TCP mode
    #[arg(
        long,
        short,
        env,
        required_if_eq("proxy_mode", "tcp")
    )]
    pub remote_address: Option<SocketAddr>,
}

#[derive(ValueEnum, Debug, Clone, Copy, Serialize)]
pub enum LogFormat {
    /// Pretty logs for debugging
    Console,
    /// JSON logs
    Json,
}

#[derive(ValueEnum, Debug, Clone, Copy, Serialize)]
pub enum Mode {
    Http,
    Socks5,
    Tcp,
}

#[derive(ValueEnum, Debug, Clone, Copy, Serialize)]
pub enum LogLevel {
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => Self::OFF,
            LogLevel::Trace => Self::TRACE,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Info => Self::INFO,
            LogLevel::Warn => Self::WARN,
            LogLevel::Error => Self::ERROR,
        }
    }
}

pub trait LogStruct {
    fn log(&self);
}

impl<T> LogStruct for T
where
    T: Serialize,
{
    fn log(&self) {
        if let Ok(json_obj) = to_value(self) {
            if let Ok(json_obj) =
                json_obj.as_object().ok_or_else(|| anyhow!("WTF"))
            {
                for (key, value) in json_obj {
                    tracing::debug!("Config {}={}", key, value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Config::command().debug_assert();
    }
}
