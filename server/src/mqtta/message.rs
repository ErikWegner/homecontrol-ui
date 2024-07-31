use std::sync::Arc;

use tokio::sync::{oneshot, watch};

pub(crate) enum ActorMessage {
    /// Query status
    Status { respond_to: oneshot::Sender<String> },
    /// Subscribe to a topic and start watching
    Subscribe {
        topic: String,
        respond_to: oneshot::Sender<watch::Receiver<Arc<String>>>,
    },
}
