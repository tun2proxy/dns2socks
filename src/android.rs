#![cfg(target_os = "android")]

use crate::{ArgProxy, ArgVerbosity, Config, LIB_NAME, main_entry};
use jni::{
    Env, EnvUnowned,
    objects::{JClass, JString},
    sys::{jboolean, jint},
};

static TUN_QUIT: std::sync::Mutex<Option<tokio_util::sync::CancellationToken>> = std::sync::Mutex::new(None);

/// # Safety
///
/// Start dns2socks
/// Parameters:
/// - listen_addr: the listen address, e.g. "172.19.0.1:53", or null to use the default value
/// - dns_remote_server: the dns remote server, e.g. "8.8.8.8:53", or null to use the default value
/// - socks5_settings: the socks5 server, e.g. "socks5://[username[:password]@]host:port", or null to use the default value
/// - force_tcp: whether to force tcp, true or false, default is false
/// - cache_records: whether to cache dns records, true or false, default is false
/// - verbosity: the verbosity level, see ArgVerbosity enum, default is ArgVerbosity::Info
/// - timeout: the timeout in seconds, default is 5
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_github_shadowsocks_bg_Dns2socks_start(
    mut env: EnvUnowned<'_>,
    _clazz: JClass<'_>,
    listen_addr: JString<'_>,
    dns_remote_server: JString<'_>,
    socks5_settings: JString<'_>,
    force_tcp: jboolean,
    cache_records: jboolean,
    verbosity: jint,
    timeout: jint,
) -> jint {
    let verbosity: ArgVerbosity = verbosity.try_into().unwrap_or_default();
    let filter_str = &format!("off,{LIB_NAME}={verbosity}");
    let filter = android_logger::FilterBuilder::new().parse(filter_str).build();
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("dns2socks")
            .with_max_level(log::LevelFilter::Trace)
            .with_filter(filter),
    );

    env.with_env(|env: &mut Env| -> Result<jint, jni::errors::Error> {
        let listen_addr = match get_java_string(env, &listen_addr) {
            Ok(addr) => addr,
            Err(_e) => "0.0.0.0:53".to_string(),
        };
        let dns_remote_server = match get_java_string(env, &dns_remote_server) {
            Ok(addr) => addr,
            Err(_e) => "8.8.8.8:53".to_string(),
        };
        let socks5_settings = match get_java_string(env, &socks5_settings) {
            Ok(addr) => addr,
            Err(_e) => "socks5://127.0.0.1:1080".to_string(),
        };
        let timeout = if timeout < 3 { 5 } else { timeout as u64 };

        let shutdown_token = tokio_util::sync::CancellationToken::new();
        if let Ok(mut lock) = TUN_QUIT.lock() {
            if lock.is_some() {
                return Ok::<jint, jni::errors::Error>(-1);
            }
            *lock = Some(shutdown_token.clone());
        } else {
            return Ok::<jint, jni::errors::Error>(-2);
        }

        let main_loop = async move {
            let mut cfg = Config::default();
            cfg.verbosity(verbosity)
                .timeout(timeout)
                .force_tcp(force_tcp)
                .cache_records(cache_records)
                .listen_addr(listen_addr.parse().map_err(std::io::Error::other)?)
                .dns_remote_server(dns_remote_server.parse()?)
                .socks5_settings(ArgProxy::try_from(socks5_settings.as_str()).map_err(std::io::Error::other)?);

            if let Err(err) = main_entry(cfg, shutdown_token).await {
                log::error!("main loop error: {}", err);
                return Err(err);
            }
            Ok(Ok::<jint, std::io::Error>(0))
        };

        match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
            Err(_e) => Ok::<jint, jni::errors::Error>(-3),
            Ok(rt) => match rt.block_on(main_loop) {
                Ok(_) => Ok::<jint, jni::errors::Error>(0),
                Err(_e) => Ok::<jint, jni::errors::Error>(-4),
            },
        }
    })
    .resolve::<jni::errors::LogErrorAndDefault>()
}

/// # Safety
///
/// Shutdown dns2socks
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_github_shadowsocks_bg_Dns2socks_stop(_env: EnvUnowned<'_>, _: JClass<'_>) -> jint {
    if let Ok(mut lock) = TUN_QUIT.lock()
        && let Some(shutdown_token) = lock.take()
    {
        shutdown_token.cancel();
        return 0;
    }
    -1
}

fn get_java_string(env: &Env, string: &JString) -> std::io::Result<String> {
    string.try_to_string(env).map_err(std::io::Error::other)
}
