use dns2socks::{main_entry, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config = Config::parse_cmd();

    dotenvy::dotenv().ok();

    let default = format!("{}={:?}", module_path!(), config.verbosity);
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default)).init();

    main_entry(config).await?;

    Ok(())
}
