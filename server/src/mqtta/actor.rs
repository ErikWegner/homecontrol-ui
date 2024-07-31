use std::{collections::HashMap, sync::Arc, time::Duration};

use rumqttc::{mqttbytes::v4::Packet::Publish, AsyncClient, Event::Incoming, MqttOptions, QoS};
use tokio::{
    sync::{mpsc, watch, RwLock},
    task,
};
use tracing::{debug, error};

use super::message::ActorMessage;

type WatcherMap =
    Arc<RwLock<HashMap<String, (watch::Sender<Arc<String>>, watch::Receiver<Arc<String>>)>>>;

pub(super) struct SubscriberActor {
    pub(crate) receiver: mpsc::Receiver<ActorMessage>,
    watchers: WatcherMap,
    client: AsyncClient,
}

impl SubscriberActor {
    pub(super) fn new(receiver: mpsc::Receiver<ActorMessage>) -> Self {
        let mut mqttoptions = MqttOptions::new("rumqtt-actor", "test.mosquitto.org", 1883);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let watchers: WatcherMap = Default::default();
        let loopmap = watchers.clone();

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        task::spawn(async move {
            debug!("Actor mqtt started");
            loop {
                let p = eventloop.poll().await;
                match p {
                    Ok(p) => {
                        debug!("Actor mqtt received = {:?}", p);
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

    pub(super) async fn handle(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::Status { respond_to } => {
                let _ = respond_to.send(String::from("implementation pending"));
            }
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
        }
    }
}
