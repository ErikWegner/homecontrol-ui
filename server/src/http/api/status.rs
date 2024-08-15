use axum::extract::State;
use jwt_authorizer::{JwtClaims, RegisteredClaims};
use tokio::sync::oneshot;
use tracing::debug;

use crate::mqtta::{message::ActorMessage, MqttHandle};

pub(crate) async fn status_handler(
    JwtClaims(user): JwtClaims<RegisteredClaims>,
    State(mqtt): State<MqttHandle>,
) -> String {
    debug!("Status request for user: {:?}", user);
    let (tx, rx) = oneshot::channel::<String>();
    tokio::spawn(async move {
        mqtt.send(ActorMessage::Status { respond_to: tx }).await;
    });

    match rx.await {
        Ok(v) => v,
        Err(_) => "No response".to_string(),
    }
}
