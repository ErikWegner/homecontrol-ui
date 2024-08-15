use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

use color_eyre::eyre::{eyre, Context, OptionExt, Result};
//allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::StreamExt};
use jwt_authorizer::{JwtClaims, RegisteredClaims};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
};
use tracing::{debug, error};

use crate::mqtta::{message::ActorMessage, MqttHandle};

enum WSIncomingMessage {
    Subscribe { topic: String },
}

pub(crate) async fn ws_handler(
    JwtClaims(user): JwtClaims<RegisteredClaims>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(mqtt): State<MqttHandle>,
) -> impl IntoResponse {
    debug!("Websocket request for user: {:?}", user);
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    debug!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, mqtt))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, who: SocketAddr, mqtt: MqttHandle) {
    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        debug!("Pinged {who}...");
    } else {
        debug!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (mut ws_client_sender, mut ws_client_receiver) = socket.split();
    let (subscription_updates_tx, mut subscription_updates_rx) = mpsc::channel::<Arc<String>>(100);

    let mut tasks: Vec<(oneshot::Sender<()>, JoinHandle<()>)> = Vec::new();
    loop {
        tokio::select! {
            // Forward incoming value updates to ws_client
            Some(v) = subscription_updates_rx.recv() => {
                    let m = Message::Text(v.to_string());
                    let _ = ws_client_sender.send(m).await;
            }
            // Process incoming websocket messages
            ws = ws_client_receiver.next() => {
                match ws {
                    None => break,
                    Some(msg) => {
                        debug!("Received message: {:?}", msg);
                        if let Ok(Message::Text(text)) = msg {
                            let m = p(&text);
                            match m {
                                Ok(m) => match m {
                                    WSIncomingMessage::Subscribe { topic } => {
                                        let (tx_subscribe, rx_subscribe) =
                                            oneshot::channel::<watch::Receiver<Arc<String>>>();
                                        let (tx_quit, mut rx_quit) = oneshot::channel::<()>();

                                        let tsubscription_updates_tx = subscription_updates_tx.clone();
                                        let tmqtt = mqtt.clone();
                                        let ttopic = topic.clone();
                                        let subscribe_task = tokio::spawn(async move {
                                            debug!(topic = ttopic, "Watcher task started");
                                            match rx_subscribe.await {
                                                Ok(mut w) => {
                                                    debug!("Received watch channel");
                                                    loop {
                                                        tokio::select! {
                                                            _ = &mut rx_quit => {
                                                                debug!("Subscribe watcher task received stop signal");
                                                                break;
                                                            }
                                                            _ = w.changed() => {
                                                                debug!("Received watch channel change");
                                                                let v = (*w.borrow_and_update()).clone();
                                                                let _ = tsubscription_updates_tx.send(Arc::new(v.to_string())).await;
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(_) => {
                                                    debug!("Could not subscribe to topic");
                                                }
                                            }
                                            debug!("Subscribe watcher task stopped");

                                        });
                                        let message = ActorMessage::Subscribe {
                                            topic,
                                            respond_to: tx_subscribe,
                                        };
                                        debug!("Sending subscribe message to MqttHandle");
                                        tmqtt.send(message).await;

                                        tasks.push((tx_quit, subscribe_task));
                                    }
                                },
                                Err(e) => error!("Invalid message {:?}", e),
                            }
                        }
                    }
                }
            }
        };
    }

    debug!("Sending stop signal to all watcher task");
    while let Some((tx_quit, subscribe_task)) = tasks.pop() {
        if tx_quit.send(()).is_ok() {
            subscribe_task.await.unwrap();
        }
    }

    // returning from the handler closes the websocket connection
    debug!("Websocket context {who} destroyed");
}

fn p(text: &str) -> Result<WSIncomingMessage> {
    let parsed =
        serde_json::from_str::<serde_json::Value>(text).wrap_err("Failed to parse JSON")?;
    let obj = parsed.as_object().ok_or_eyre("Must be an object")?;
    let mb_command = obj
        .get("cmd")
        .ok_or_eyre("Missing command")?
        .as_str()
        .ok_or_eyre("Command must be a string")?;
    let mb_topic = obj
        .get("topic")
        .ok_or_eyre("Missing topic")?
        .as_str()
        .ok_or_eyre("Topic must be a string")?;
    match mb_command {
        "sub" => Ok(WSIncomingMessage::Subscribe {
            topic: mb_topic.to_string(),
        }),
        _ => Err(eyre!("Unknown command: {mb_command}")),
    }
}
