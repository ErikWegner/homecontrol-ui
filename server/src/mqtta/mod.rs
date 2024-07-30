mod actor;
mod handle;
mod message;

use std::{sync::Arc, time::Duration};

use actor::SubscriberActor;
use handle::MqttHandle;
use message::ActorMessage;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task, time,
};
use tracing::debug;

async fn run_subscriber_actor() -> (MqttHandle, oneshot::Sender<()>) {
    debug!("run_subscriber_actor");
    let (sender, receiver) = mpsc::channel(8);
    let mut actor = SubscriberActor::new(receiver);
    let (tx, mut rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        loop {
            debug!("Loop actor receiver");
            tokio::select! {
                _ = &mut rx => {
                    break;
                }
                Some(msg) = actor.receiver.recv() => {
                    debug!("Actor message received");
                    actor.handle(msg).await;
                }
            }
        }
    });
    let handle = MqttHandle::new(sender);
    (handle, tx)
}

pub(crate) async fn mqtta() {
    debug!("Creating actor");
    let (actor, stopmqttsignal) = run_subscriber_actor().await;

    debug!("Subscribing");
    let (wtx, wrx) = oneshot::channel::<watch::Receiver<Arc<String>>>();
    task::spawn(async move {
        debug!("Starting update watcher");
        match wrx.await {
            Ok(mut w) => {
                debug!("Watchchannel received");
                loop {
                    let v = (*w.borrow_and_update()).clone();
                    debug!("Watchchannel received: {:?}", v);
                    if w.changed().await.is_err() {
                        debug!("Leaving watchchannel loop");
                        break;
                    }
                }
            }
            Err(_) => {
                debug!("No channel received");
            }
        }
    });
    let m = ActorMessage::Subscribe {
        topic: String::from("hello/rumqtt"),
        respond_to: wtx,
    };
    actor.send(m).await;

    let (donetx, donerx) = oneshot::channel::<()>();

    debug!("Sending updates");
    task::spawn(async move {
        let mut mqttoptions = MqttOptions::new("rumqtt-publ", "test.mosquitto.org", 1883);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        for i in 0..10 {
            debug!("Publishing: {}", i);
            client
                .publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize])
                .await
                .unwrap();
            time::sleep(Duration::from_millis(333)).await;
            if let Ok(notification) = eventloop.poll().await {
                println!("Sending client received = {:?}", notification);
            }
        }
        donetx.send(()).unwrap();
    });

    let _ = donerx.await;
    debug!("Stopping");
    let _ = stopmqttsignal.send(());
    debug!("Stopped");
}
