mod api;
pub(crate) mod appstate;

use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use api::{status::status_handler, web2mqtt::web2mqtt_handler, ws::ws_handler};
use appstate::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use color_eyre::{eyre::Context, Result};
use jwt_authorizer::{Authorizer, IntoLayer, JwtAuthorizer, Validation};
use tokio::signal;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing::debug;

async fn api_routes(state: AppState) -> Result<Router> {
    let url = std::env::var("HCS_JWT_ISSUER").wrap_err("Missing HCS_JWT_ISSUER variable")?;
    let validation = Validation::new()
        .iss(&[url.clone()])
        .aud(&["homecontrol"])
        .leeway(5);
    let auth: Authorizer = JwtAuthorizer::from_oidc(&url)
        .validation(validation)
        .build()
        .await
        .wrap_err("JWT authorization initialization failed")?;
    Ok(Router::new()
        .route("/status", get(status_handler))
        .route("/publish", post(web2mqtt_handler))
        .route("/ws", get(ws_handler))
        .layer(auth.into_layer())
        .with_state(state))
}

pub(crate) async fn http_server(state: AppState) -> Result<()> {
    let app = Router::new().nest("/api", api_routes(state).await?).layer((
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

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
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
