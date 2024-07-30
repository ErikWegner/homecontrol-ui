use tokio::sync::mpsc;

use super::message::ActorMessage;

#[derive(Clone)]
pub(crate) struct MqttHandle {
    sender: mpsc::Sender<ActorMessage>,
}

impl MqttHandle {
    pub(super) fn new(sender: mpsc::Sender<ActorMessage>) -> Self {
        Self { sender }
    }

    pub(crate) async fn send(&self, message: ActorMessage) {
        let _ = self.sender.send(message).await;
    }
}
