use std::net::SocketAddr;

/// Proxy server to routing DNS query to SOCKS5 server
#[derive(clap::Parser, Debug, Clone, PartialEq, Eq)]
#[command(author, version, about = "Proxy server to routing DNS query to SOCKS5 server", long_about = None)]
pub struct Config {
    /// Listen address
    #[clap(short, long, value_name = "IP:port", default_value = "0.0.0.0:53")]
    pub listen_addr: SocketAddr,

    /// Remote DNS server address
    #[clap(short, long, value_name = "IP:port", default_value = "8.8.8.8:53")]
    pub dns_remote_server: SocketAddr,

    /// SOCKS5 proxy server address
    #[clap(short, long, value_name = "IP:port", default_value = "127.0.0.1:1080")]
    pub socks5_server: SocketAddr,

    /// User name for SOCKS5 authentication
    #[clap(short, long, value_name = "user name")]
    pub username: Option<String>,

    /// Password for SOCKS5 authentication
    #[clap(short, long, value_name = "password")]
    pub password: Option<String>,

    /// Force to use TCP to proxy DNS query
    #[clap(short, long)]
    pub force_tcp: bool,

    /// Cache DNS query records
    #[clap(short, long)]
    pub cache_records: bool,

    /// Verbosity level
    #[arg(short, long, value_name = "level", value_enum, default_value = "info")]
    pub verbosity: ArgVerbosity,

    /// Timeout for DNS query
    #[clap(short, long, value_name = "seconds", default_value = "5")]
    pub timeout: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            listen_addr: "0.0.0.0:53".parse().unwrap(),
            dns_remote_server: "8.8.8.8:53".parse().unwrap(),
            socks5_server: "127.0.0.1:1080".parse().unwrap(),
            username: None,
            password: None,
            force_tcp: false,
            cache_records: false,
            verbosity: ArgVerbosity::default(),
            timeout: 5,
        }
    }
}

impl Config {
    pub fn parse_cmd() -> Self {
        clap::Parser::parse()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Default)]
pub enum ArgVerbosity {
    Off = 0,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for ArgVerbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgVerbosity::Off => write!(f, "off"),
            ArgVerbosity::Error => write!(f, "error"),
            ArgVerbosity::Warn => write!(f, "warn"),
            ArgVerbosity::Info => write!(f, "info"),
            ArgVerbosity::Debug => write!(f, "debug"),
            ArgVerbosity::Trace => write!(f, "trace"),
        }
    }
}
