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
    pub fn parse_args() -> Self {
        clap::Parser::parse()
    }

    pub fn listen_addr(&mut self, listen_addr: SocketAddr) -> &mut Self {
        self.listen_addr = listen_addr;
        self
    }

    pub fn dns_remote_server(&mut self, dns_remote_server: SocketAddr) -> &mut Self {
        self.dns_remote_server = dns_remote_server;
        self
    }

    pub fn socks5_server(&mut self, socks5_server: SocketAddr) -> &mut Self {
        self.socks5_server = socks5_server;
        self
    }

    pub fn username(&mut self, username: Option<String>) -> &mut Self {
        self.username = username;
        self
    }

    pub fn password(&mut self, password: Option<String>) -> &mut Self {
        self.password = password;
        self
    }

    pub fn force_tcp(&mut self, force_tcp: bool) -> &mut Self {
        self.force_tcp = force_tcp;
        self
    }

    pub fn cache_records(&mut self, cache_records: bool) -> &mut Self {
        self.cache_records = cache_records;
        self
    }

    pub fn verbosity(&mut self, verbosity: ArgVerbosity) -> &mut Self {
        self.verbosity = verbosity;
        self
    }

    pub fn timeout(&mut self, timeout: u64) -> &mut Self {
        self.timeout = timeout;
        self
    }
}

#[repr(C)]
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

impl From<log::Level> for ArgVerbosity {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Error => ArgVerbosity::Error,
            log::Level::Warn => ArgVerbosity::Warn,
            log::Level::Info => ArgVerbosity::Info,
            log::Level::Debug => ArgVerbosity::Debug,
            log::Level::Trace => ArgVerbosity::Trace,
        }
    }
}

impl From<ArgVerbosity> for log::LevelFilter {
    fn from(level: ArgVerbosity) -> Self {
        match level {
            ArgVerbosity::Off => log::LevelFilter::Off,
            ArgVerbosity::Error => log::LevelFilter::Error,
            ArgVerbosity::Warn => log::LevelFilter::Warn,
            ArgVerbosity::Info => log::LevelFilter::Info,
            ArgVerbosity::Debug => log::LevelFilter::Debug,
            ArgVerbosity::Trace => log::LevelFilter::Trace,
        }
    }
}

impl TryFrom<i32> for ArgVerbosity {
    type Error = std::io::Error;

    fn try_from(value: i32) -> Result<Self, <ArgVerbosity as TryFrom<i32>>::Error> {
        match value {
            0 => Ok(ArgVerbosity::Off),
            1 => Ok(ArgVerbosity::Error),
            2 => Ok(ArgVerbosity::Warn),
            3 => Ok(ArgVerbosity::Info),
            4 => Ok(ArgVerbosity::Debug),
            5 => Ok(ArgVerbosity::Trace),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid verbosity level")),
        }
    }
}
