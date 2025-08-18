use dns2socks::{Config, main_entry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config = Config::parse_args();

    dotenvy::dotenv().ok();

    let default = format!("{}={:?}", module_path!(), config.verbosity);
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default)).init();

    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let join_handle = tokio::spawn({
        let shutdown_token = shutdown_token.clone();
        async move {
            if let Err(err) = main_entry(config, shutdown_token).await {
                log::error!("main loop error: {}", err);
            }
        }
    });

    let async_ctrlc = ctrlc2::AsyncCtrlC::new(move || {
        log::info!("Ctrl-C received, exiting...");
        shutdown_token.cancel();
        true
    })?;

    if let Err(err) = join_handle.await {
        log::error!("main_entry error {}", err);
    }

    tokio::time::timeout(std::time::Duration::from_millis(100), async_ctrlc).await??;

    Ok(())
}
