use crate::{main_entry, ArgVerbosity};
use std::ffi::{c_char, c_int};

static TUN_QUIT: std::sync::Mutex<Option<tokio_util::sync::CancellationToken>> = std::sync::Mutex::new(None);

/// # Safety
///
/// Run the dns2socks component with some arguments.
/// Parameters:
/// - listen_addr: the listen address, e.g. "0.0.0.0:53", or null to use the default value
/// - dns_remote_server: the dns remote server, e.g. "8.8.8.8:53", or null to use the default value
/// - socks5_server: the socks5 server, e.g. "127.0.0.1:1080", or null to use the default value
/// - username: the username for socks5 authentication, or null to use the default value
/// - password: the password for socks5 authentication, or null to use the default value
/// - force_tcp: whether to force tcp, true or false, default is false
/// - cache_records: whether to cache dns records, true or false, default is false
/// - verbosity: the verbosity level, see ArgVerbosity enum, default is ArgVerbosity::Info
/// - timeout: the timeout in seconds, default is 5
#[no_mangle]
pub unsafe extern "C" fn dns2socks_start(
    listen_addr: *const c_char,
    dns_remote_server: *const c_char,
    socks5_server: *const c_char,
    username: *const c_char,
    password: *const c_char,
    force_tcp: bool,
    cache_records: bool,
    verbosity: ArgVerbosity,
    timeout: i32,
) -> c_int {
    let shutdown_token = tokio_util::sync::CancellationToken::new();
    {
        if let Ok(mut lock) = TUN_QUIT.lock() {
            if lock.is_some() {
                return -1;
            }
            *lock = Some(shutdown_token.clone());
        } else {
            return -2;
        }
    }

    log::set_max_level(verbosity.into());
    if let Err(err) = log::set_boxed_logger(Box::<crate::dump_logger::DumpLogger>::default()) {
        log::error!("set logger error: {}", err);
        return -1;
    }

    let mut config = crate::Config::default();
    config
        .verbosity(verbosity)
        .timeout(timeout as u64)
        .force_tcp(force_tcp)
        .cache_records(cache_records);
    if !listen_addr.is_null() {
        let listen_addr = std::ffi::CStr::from_ptr(listen_addr).to_str().unwrap();
        config.listen_addr(listen_addr.parse().unwrap());
    }
    if !dns_remote_server.is_null() {
        let dns_remote_server = std::ffi::CStr::from_ptr(dns_remote_server).to_str().unwrap();
        config.dns_remote_server(dns_remote_server.parse().unwrap());
    }
    if !socks5_server.is_null() {
        let socks5_server = std::ffi::CStr::from_ptr(socks5_server).to_str().unwrap();
        config.socks5_server(socks5_server.parse().unwrap());
    }
    if !username.is_null() {
        let username = std::ffi::CStr::from_ptr(username).to_str().unwrap().to_string();
        config.username(Some(username));
    }
    if !password.is_null() {
        let password = std::ffi::CStr::from_ptr(password).to_str().unwrap().to_string();
        config.password(Some(password));
    }

    let main_loop = async move {
        if let Err(err) = main_entry(config, shutdown_token).await {
            log::error!("main loop error: {}", err);
            return Err(err);
        }
        Ok(())
    };

    let exit_code = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Err(_e) => -3,
        Ok(rt) => match rt.block_on(main_loop) {
            Ok(_) => 0,
            Err(_e) => -4,
        },
    };

    exit_code
}

/// # Safety
///
/// Shutdown the dns2socks component.
#[no_mangle]
pub unsafe extern "C" fn dns2socks_stop() -> c_int {
    if let Ok(mut lock) = TUN_QUIT.lock() {
        if let Some(shutdown_token) = lock.take() {
            shutdown_token.cancel();
            return 0;
        }
    }
    -1
}
