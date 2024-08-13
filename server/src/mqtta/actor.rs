use std::{collections::HashMap, env, sync::Arc, time::Duration};

use color_eyre::eyre::{eyre, Context, Result};
use rand::distributions::{Alphanumeric, DistString};
use rumqttc::{mqttbytes::v4::Packet::Publish, AsyncClient, Event::Incoming, MqttOptions, QoS};
use tokio::{
    sync::{mpsc, watch, RwLock},
    task,
};
use tracing::{debug, error, info, warn};

use super::message::ActorMessage;

type WatcherMap =
    Arc<RwLock<HashMap<String, (watch::Sender<Arc<String>>, watch::Receiver<Arc<String>>)>>>;

pub(super) struct SubscriberActor {
    pub(crate) receiver: mpsc::Receiver<ActorMessage>,
    watchers: WatcherMap,
    client: AsyncClient,
    run: Arc<RwLock<bool>>,
    polltask: task::JoinHandle<()>,
}

fn mqtt_client_id() -> Result<String> {
    if let Ok(c) = env::var("HCS_MQTT_CLIENT_ID") {
        if c.is_empty() {
            return Err(eyre!("HCS_MQTT_CLIENT_ID cannot be empty"));
        }
        return Ok(c);
    }

    let mut hostname = String::from("client");
    if let Ok(h) = hostname::get() {
        if let Ok(s) = h.into_string() {
            hostname = s;
        }
    }

    let randompart = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);

    Ok(format!("hcs-{hostname}-{randompart}"))
}

fn mqtt_host() -> Result<String> {
    let mqtt_host = env::var("HCS_MQTT_HOST");
    if let Ok(h) = mqtt_host {
        return Ok(h);
    }

    let hostname = String::from("test.mosquitto.org");
    warn!("Using MQTT host: {hostname}");

    Ok(hostname)
}

fn mqtt_port() -> Result<u16> {
    let mqtt_port = env::var("HCS_MQTT_PORT");
    if let Ok(p) = mqtt_port {
        return p.parse::<u16>().wrap_err_with(|| "Invalid MQTT port");
    }

    Ok(1883)
}

fn mqtt_credentials(mo: &mut MqttOptions) {
    if let Ok(username) = env::var("HCS_MQTT_USERNAME") {
        if !username.is_empty() {
            if let Ok(password) = env::var("HCS_MQTT_PASSWORD") {
                if !password.is_empty() {
                    mo.set_credentials(username, password);
                }
            }
        }
    }
}

fn mqtt_keepalive() -> Result<u64> {
    if let Ok(ka) = env::var("HCS_MQTT_KEEPALIVE") {
        return ka.parse::<u64>().wrap_err_with(|| "Invalid MQTT keepalive");
    }

    Ok(15)
}

pub(crate) fn mqtt_options_from_env() -> color_eyre::Result<MqttOptions> {
    let mqtt_id = mqtt_client_id()?;
    let mqtt_host = mqtt_host()?;
    let mqtt_port = mqtt_port()?;
    let mut mqttoptions = MqttOptions::new(mqtt_id, mqtt_host, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(mqtt_keepalive()?));
    mqtt_credentials(&mut mqttoptions);
    Ok(mqttoptions)
}

impl SubscriberActor {
    pub(super) fn new(receiver: mpsc::Receiver<ActorMessage>, mqttoptions: MqttOptions) -> Self {
        debug!("Creating subscriber actor");
        let watchers: WatcherMap = Default::default();
        let loopmap = watchers.clone();
        let runindicator = Arc::new(RwLock::new(true));
        let runloopindicator = runindicator.clone();

        let (hostname, port) = mqttoptions.broker_address();
        let clientid = mqttoptions.client_id();
        let with_credentials = mqttoptions.credentials().is_some();
        info!(hostname, port, clientid, with_credentials, "Using mqtt");
        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        let polltask = task::spawn(async move {
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
                    }
                }
                let keeprunning = runloopindicator.read().await;
                if !(*keeprunning) {
                    debug!("Actor mqtt can stop now");
                    break;
                }
            }
            debug!("Actor mqtt stopped");
        });
        SubscriberActor {
            receiver,
            watchers,
            client,
            run: runindicator,
            polltask,
        }
    }

    pub(super) async fn handle(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::Publish {
                payload,
                respond_to,
            } => {
                let pubresult = self
                    .client
                    .publish(
                        &payload.topic,
                        payload.qos,
                        payload.retain,
                        payload.value.clone(),
                    )
                    .await;
                let _ = respond_to.send(match pubresult {
                    Ok(_) => String::from("OK"),
                    Err(err) => {
                        warn!("Sending {:?} failed {:?}", payload, err);
                        String::from("Error")
                    }
                });
            }
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

    pub(crate) async fn stop(self) {
        debug!("Setting stop signal");
        {
            let mut r = self.run.write().await;
            *r = false;
        }

        debug!("Waiting for event loop task to finish");
        let polltaskresult = self.polltask.await;
        if let Err(polltaskerr) = polltaskresult {
            error!("Failed to stop polling task, {:?}", polltaskerr);
        }
    }
}
