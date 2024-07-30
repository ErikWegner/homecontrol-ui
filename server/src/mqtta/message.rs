use std::sync::Arc;

use tokio::sync::{oneshot, watch};

pub(crate) enum ActorMessage {
    Subscribe {
        topic: String,
        respond_to: oneshot::Sender<watch::Receiver<Arc<String>>>,
    },
    Shutdown,
}
