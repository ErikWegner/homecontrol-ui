use axum::extract::State;
use color_eyre::owo_colors::OwoColorize;
use rumqttc::QoS;
use serde::{de::value::U8Deserializer, Deserialize};
use tokio::sync::oneshot;

use crate::mqtta::{message::ActorMessage, MqttHandle};

#[derive(Deserialize)]
struct Web2MqttRequestBody {
    pub topic: String,
    pub value: String,
    pub qos: QoS,
}

pub(crate) async fn web2mqtt(State(mqtt): State<MqttHandle>) -> String {
    let (tx, rx) = oneshot::channel::<String>();
    tokio::spawn(async move {
        mqtt.send(ActorMessage::Status { respond_to: tx }).await;
    });

    match rx.await {
        Ok(v) => v,
        Err(_) => "No response".to_string(),
    }
}
