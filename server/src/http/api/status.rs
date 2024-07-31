use axum::extract::State;
use tokio::sync::oneshot;

use crate::mqtta::{message::ActorMessage, MqttHandle};

pub(crate) async fn status_handler(State(mqtt): State<MqttHandle>) -> String {
    let (tx, rx) = oneshot::channel::<String>();
    tokio::spawn(async move {
        mqtt.send(ActorMessage::Status { respond_to: tx }).await;
    });

    match rx.await {
        Ok(v) => v,
        Err(_) => "No response".to_string(),
    }
}
