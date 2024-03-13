# dns2socks

[![Crates.io](https://img.shields.io/crates/v/dns2socks.svg)](https://crates.io/crates/dns2socks)
![dns2socks](https://docs.rs/dns2socks/badge.svg)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/dns2socks)
[![Download](https://img.shields.io/crates/d/dns2socks.svg)](https://crates.io/crates/dns2socks)
[![License](https://img.shields.io/crates/l/dns2socks.svg?style=flat)](https://github.com/ssrlive/dns2socks/blob/master/LICENSE)

A DNS server that forwards DNS requests to a SOCKS5 server.

## Installation

### Precompiled Binaries

Download binary from [releases](https://github.com/ssrlive/dns2socks/releases) and put it in your `$PATH`.

### Install from Crates.io

If you have [Rust](https://rustup.rs/) toolchain installed, you can install `dns2socks` with the following command:
```sh
cargo install dns2socks
```

## Usage

```plaintext
dns2socks -h

Proxy server to routing DNS query to SOCKS5 server

Usage: dns2socks [OPTIONS]

Options:
  -l, --listen-addr <IP:port>        Listen address [default: 0.0.0.0:53]
  -d, --dns-remote-server <IP:port>  Remote DNS server address [default: 8.8.8.8:53]
  -s, --socks5-server <IP:port>      SOCKS5 proxy server address [default: 127.0.0.1:1080]
  -u, --username <user name>         User name for SOCKS5 authentication
  -p, --password <password>          Password for SOCKS5 authentication
  -f, --force-tcp                    Force to use TCP to proxy DNS query
  -c, --cache-records                Cache DNS query records
  -v, --verbosity <level>            Verbosity level [default: info] [possible values: off, error, warn, info, debug, trace]
  -t, --timeout <seconds>            Timeout for DNS query [default: 5]
  -h, --help                         Print help
  -V, --version                      Print version
```
