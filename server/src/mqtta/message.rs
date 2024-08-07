use std::sync::Arc;

use rumqttc::QoS;
use tokio::sync::{oneshot, watch};
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub(crate) struct PublishMessage {
    pub(crate) topic: String,
    pub(crate) value: Vec<u8>,
    pub(crate) qos: QoS,
    pub(crate) retain: bool,
}

pub(crate) enum ActorMessage {
    /// Publish
    Publish {
        payload: PublishMessage,
        respond_to: oneshot::Sender<String>,
    },
    /// Query status
    Status { respond_to: oneshot::Sender<String> },
    /// Subscribe to a topic and start watching
    Subscribe {
        topic: String,
        respond_to: oneshot::Sender<watch::Receiver<Arc<String>>>,
    },
}
