use socks5_impl::protocol::UserKey;
use std::net::{SocketAddr, ToSocketAddrs as _};

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

    /// SOCKS5 URL in the form socks5://[username[:password]@]host:port,
    /// Username and password are encoded in percent encoding. For example:
    /// socks5://myname:pass%40word@127.0.0.1:1080
    #[arg(short, long, value_parser = |s: &str| ArgProxy::try_from(s), value_name = "URL", default_value = "socks5://127.0.0.1:1080")]
    pub socks5_settings: ArgProxy,

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
            socks5_settings: ArgProxy::default(),
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

    pub fn socks5_settings(&mut self, socks5_settings: ArgProxy) -> &mut Self {
        self.socks5_settings = socks5_settings;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgProxy {
    pub proxy_type: ProxyType,
    pub addr: SocketAddr,
    pub credentials: Option<UserKey>,
}

impl Default for ArgProxy {
    fn default() -> Self {
        ArgProxy {
            proxy_type: ProxyType::Socks5,
            addr: "127.0.0.1:1080".parse().unwrap(),
            credentials: None,
        }
    }
}

impl std::fmt::Display for ArgProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let auth = match &self.credentials {
            Some(creds) => format!("{}", creds),
            None => "".to_owned(),
        };
        if auth.is_empty() {
            write!(f, "{}://{}", &self.proxy_type, &self.addr)
        } else {
            write!(f, "{}://{}@{}", &self.proxy_type, auth, &self.addr)
        }
    }
}

impl TryFrom<&str> for ArgProxy {
    type Error = std::io::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        use std::io::{Error, ErrorKind::InvalidData};
        let e = format!("`{s}` is not a valid proxy URL");
        let url = url::Url::parse(s).map_err(|_| Error::new(InvalidData, e))?;
        let e = format!("`{s}` does not contain a host");
        let host = url.host_str().ok_or(Error::new(InvalidData, e))?;

        let e = format!("`{s}` does not contain a port");
        let port = url.port_or_known_default().ok_or(Error::new(InvalidData, e))?;

        let e2 = format!("`{host}` does not resolve to a usable IP address");
        let addr = (host, port).to_socket_addrs()?.next().ok_or(Error::new(InvalidData, e2))?;

        let credentials = if url.username() == "" && url.password().is_none() {
            None
        } else {
            let username = percent_encoding::percent_decode(url.username().as_bytes())
                .decode_utf8()
                .map_err(|e| Error::new(InvalidData, e))?;
            let password = percent_encoding::percent_decode(url.password().unwrap_or("").as_bytes())
                .decode_utf8()
                .map_err(|e| Error::new(InvalidData, e))?;
            Some(UserKey::new(username, password))
        };

        let proxy_type = url.scheme().to_ascii_lowercase().as_str().try_into()?;

        Ok(ArgProxy {
            proxy_type,
            addr,
            credentials,
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum ProxyType {
    // Http = 0,
    // Socks4,
    #[default]
    Socks5,
}

impl TryFrom<&str> for ProxyType {
    type Error = std::io::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use std::io::{Error, ErrorKind::InvalidData};
        match value {
            // "http" => Ok(ProxyType::Http),
            // "socks4" => Ok(ProxyType::Socks4),
            "socks5" => Ok(ProxyType::Socks5),
            scheme => Err(Error::new(InvalidData, format!("`{scheme}` is an invalid proxy type"))),
        }
    }
}

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ProxyType::Http => write!(f, "http"),
            // ProxyType::Socks4 => write!(f, "socks4"),
            ProxyType::Socks5 => write!(f, "socks5"),
        }
    }
}
