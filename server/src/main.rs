use std::path::Path;

use color_eyre::eyre::Result;
use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let env_envfile = std::env::var("HCS_ENV_FILE");
    if let Ok(envfile) = env_envfile {
        let p = Path::new(&envfile);
        dotenvy::from_path(p)?;
    } else {
        let _ = dotenv();
    };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "homecontrol_ui_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("start");
    homecontrol_ui_server::run().await?;
    info!("end");
    Ok(())
}
