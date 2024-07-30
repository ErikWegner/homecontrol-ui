use crate::mqtta::mqtta;
use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod mqtta;

#[tokio::main]
async fn main() {
    let _ = dotenv();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "homecontrol_ui_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("start");
    mqtta().await;
    info!("end");
}
