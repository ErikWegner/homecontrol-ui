pub(crate) mod appstate;

use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use appstate::AppState;
use axum::{extract::State, routing::get, Router};
use color_eyre::{eyre::Context, Result};
use tokio::{signal, sync::oneshot};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing::debug;

use crate::mqtta::{message::ActorMessage, MqttHandle};

fn api_routes(state: AppState) -> Result<Router> {
    Ok(Router::new()
        .route("/status", get(status))
        .with_state(state))
}

pub(crate) async fn http_server(state: AppState) -> Result<()> {
    let app = Router::new().nest("/api", api_routes(state)?).layer((
        TraceLayer::new_for_http(),
        TimeoutLayer::new(Duration::from_secs(10)),
    ));
    debug!("Initializing service...");
    // run it
    let addr = SocketAddr::new(
        IpAddr::from_str("::")?,
        std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .context("Cannot parse PORT")?,
    );

    tracing::info!("listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Cannot start server")?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("error running server")?;

    debug!("Shutdown completed");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn status(State(mqtt): State<MqttHandle>) -> String {
    let (tx, rx) = oneshot::channel::<String>();
    tokio::spawn(async move {
        mqtt.send(ActorMessage::Status { respond_to: tx }).await;
    });

    match rx.await {
        Ok(v) => v,
        Err(_) => "No response".to_string(),
    }
}
