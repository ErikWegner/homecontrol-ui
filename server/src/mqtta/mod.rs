use std::{collections::HashMap, sync::Arc, time::Duration};

use rumqttc::{mqttbytes::v4::Packet::Publish, AsyncClient, Event::Incoming, MqttOptions, QoS};
use tokio::{
    sync::{mpsc, oneshot, watch, RwLock},
    task, time,
};
use tracing::{debug, error};

pub(crate) enum ActorMessage {
    Subscribe {
        topic: String,
        respond_to: oneshot::Sender<watch::Receiver<Arc<String>>>,
    },
    Shutdown,
}

struct SubscriberActor {
    receiver: mpsc::Receiver<ActorMessage>,
    watchers:
        Arc<RwLock<HashMap<String, (watch::Sender<Arc<String>>, watch::Receiver<Arc<String>>)>>>,
    client: AsyncClient,
}

#[derive(Clone)]
pub(crate) struct MqttHandle {
    sender: mpsc::Sender<ActorMessage>,
}

impl MqttHandle {
    fn new(sender: mpsc::Sender<ActorMessage>) -> Self {
        Self { sender }
    }

    pub(crate) async fn send(&self, message: ActorMessage) {
        let _ = self.sender.send(message).await;
    }
}

impl SubscriberActor {
    fn new(receiver: mpsc::Receiver<ActorMessage>) -> Self {
        let mut mqttoptions = MqttOptions::new("rumqtt-actor", "test.mosquitto.org", 1883);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let watchers: Arc<
            RwLock<HashMap<String, (watch::Sender<Arc<String>>, watch::Receiver<Arc<String>>)>>,
        > = Default::default();
        let loopmap = watchers.clone();

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        task::spawn(async move {
            debug!("Actor mqtt started");
            loop {
                let p = eventloop.poll().await;
                match p {
                    Ok(p) => {
                        println!("Actor mqtt received = {:?}", p);
                        if let Incoming(i) = p {
                            match i {
                                Publish(p) => {
                                    let topic = p.topic;
                                    let map = loopmap.read().await;
                                    if let Some(w) = map.get(&topic) {
                                        let tx = w.0.clone();
                                        let new_message = Arc::new(
                                            String::from_utf8(p.payload.to_vec())
                                                .unwrap_or_default(),
                                        );
                                        if tx.send(new_message).is_err() {
                                            error!("Error sending message to watcher: {:?}", topic);
                                        }
                                    } else {
                                        debug!("No watcher for topic: {}", &topic);
                                    }
                                }
                                _ => {
                                    debug!("No match for Incoming packet");
                                }
                            }
                        } else {
                            debug!("Not an incoming packet");
                        }
                    }
                    Err(e) => {
                        error!("Error polling: {:?}", e);
                        break;
                    }
                }
            }
            debug!("Actor mqtt stopped");
        });
        SubscriberActor {
            receiver,
            watchers,
            client,
        }
    }

    async fn handle(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::Subscribe { topic, respond_to } => {
                let mut w = self.watchers.write().await;
                let vo = w.get(&topic);
                let rx = if let Some(v) = vo {
                    v.1.clone()
                } else {
                    let (tx, rx) = watch::channel(Arc::new(String::new()));
                    let rrx = rx.clone();
                    w.insert(topic.clone(), (tx, rx));
                    rrx
                };
                debug!("Subscribing to: {}", &topic);
                let s = self.client.subscribe(&topic, QoS::AtMostOnce).await;
                match s {
                    Ok(_) => debug!("Subscribed to: {}", &topic),
                    Err(e) => error!("Error subscribing to: {} - {:?}", topic, e),
                }
                let _ = respond_to.send(rx);
            }
            ActorMessage::Shutdown => {
                let _ = self.client.disconnect().await;
            }
        }
    }
}

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
