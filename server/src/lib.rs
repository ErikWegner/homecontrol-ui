use color_eyre::eyre::{Context, Result};
use http::appstate::AppState;
use mqtta::run_subscriber_actor;
use tracing::debug;

mod http;
mod mqtta;

pub async fn run() -> Result<()> {
    let channelsize = std::env::var("HCS_PERF_CHANNELBUFSIZE")
        .unwrap_or_else(|_| "8".to_string())
        .parse::<usize>()
        .context("Cannot parse HCS_PERF_CHANNELBUFSIZE")?;
    let (handle, tx, jh) = run_subscriber_actor(channelsize).await;
    let appstate = AppState::builder().mqtt(handle).build();
    http::http_server(appstate).await?;
    debug!("Shutdown");
    let _ = tx.send(());
    jh.await.context("Failed to wait for mqtt shutdown")
}
