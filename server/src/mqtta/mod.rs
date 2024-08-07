mod actor;
mod handle;
pub(crate) mod message;

use actor::SubscriberActor;
pub(crate) use handle::MqttHandle;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tracing::debug;

pub(crate) async fn run_subscriber_actor(
    channelsize: usize,
) -> (MqttHandle, oneshot::Sender<()>, JoinHandle<()>) {
    debug!("Setup mqtt with {channelsize} buffer size");
    let (sender, receiver) = mpsc::channel(channelsize);
    let mut actor = SubscriberActor::new(receiver);
    let (tx, mut rx) = oneshot::channel::<()>();
    let jh = tokio::spawn(async move {
        loop {
            debug!("Loop actor receiver");
            tokio::select! {
                _ = &mut rx => {
                    debug!("Loop actor stop signal received");
                    actor.stop().await;
                    break;
                }
                Some(msg) = actor.receiver.recv() => {
                    debug!("Actor message received");
                    actor.handle(msg).await;
                }
            }
        }
        debug!("Leaving loop actor");
    });
    let handle = MqttHandle::new(sender);
    (handle, tx, jh)
}
