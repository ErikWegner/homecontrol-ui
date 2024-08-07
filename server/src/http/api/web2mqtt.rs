use axum::{debug_handler, extract::State, Json};
use rumqttc::QoS;
use serde::Deserialize;
use tokio::sync::oneshot;

use crate::mqtta::{
    message::{ActorMessage, PublishMessage},
    MqttHandle,
};

#[derive(Deserialize)]
pub(crate) struct Web2MqttRequestBody {
    pub topic: String,
    pub value: String,
    pub qos: u8,
    pub retain: bool,
}

#[debug_handler]
pub(crate) async fn web2mqtt_handler(
    State(mqtt): State<MqttHandle>,
    Json(payload): Json<Web2MqttRequestBody>,
) -> String {
    let payload = PublishMessage::builder()
        .topic(payload.topic.clone())
        .value(payload.value.clone().into_bytes())
        .qos(match payload.qos {
            2 => QoS::ExactlyOnce,
            1 => QoS::AtLeastOnce,
            _ => QoS::AtMostOnce,
        })
        .retain(payload.retain)
        .build();
    let (tx, rx) = oneshot::channel::<String>();
    tokio::spawn(async move {
        mqtt.send(ActorMessage::Publish {
            payload,
            respond_to: tx,
        })
        .await;
    });

    match rx.await {
        Ok(v) => v,
        Err(_) => "No response".to_string(),
    }
}
