use dns2socks::{main_entry, Config};

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

    ctrlc2::set_async_handler(async move {
        log::info!("Ctrl-C received, exiting...");
        shutdown_token.cancel();
    })
    .await;

    if let Err(err) = join_handle.await {
        log::error!("main_entry error {}", err);
    }

    Ok(())
}
