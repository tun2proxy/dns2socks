use socks5_impl::ProxyParameters;
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

    /// SOCKS5 URL in the form socks5://[username[:password]@]host:port,
    /// Username and password are encoded in percent encoding. For example:
    /// socks5://myname:pass%40word@127.0.0.1:1080
    #[arg(short, long, value_name = "URL", default_value = "socks5://127.0.0.1:1080")]
    pub socks5_settings: ProxyParameters,

    /// Force to use TCP to proxy DNS query
    #[clap(short, long)]
    pub force_tcp: bool,

    /// Cache DNS query records
    #[clap(short, long)]
    pub cache_records: bool,

    /// Verbosity level
    #[arg(short, long, value_name = "level", value_enum, default_value = "info")]
    pub verbosity: log::LevelFilter,

    /// Timeout for DNS query
    #[clap(short, long, value_name = "seconds", default_value = "5")]
    pub timeout: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            listen_addr: "0.0.0.0:53".parse().unwrap(),
            dns_remote_server: "8.8.8.8:53".parse().unwrap(),
            socks5_settings: ProxyParameters::default(),
            force_tcp: false,
            cache_records: false,
            verbosity: log::LevelFilter::Info,
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

    pub fn socks5_settings(&mut self, socks5_settings: &ProxyParameters) -> &mut Self {
        self.socks5_settings = socks5_settings.clone();
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

    pub fn verbosity(&mut self, verbosity: log::LevelFilter) -> &mut Self {
        self.verbosity = verbosity;
        self
    }

    pub fn timeout(&mut self, timeout: u64) -> &mut Self {
        self.timeout = timeout;
        self
    }
}
