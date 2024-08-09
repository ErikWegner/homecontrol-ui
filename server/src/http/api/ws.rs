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

//allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::{oneshot, watch};
use tracing::debug;

use crate::mqtta::{message::ActorMessage, MqttHandle};

pub(crate) async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(mqtt): State<MqttHandle>,
) -> impl IntoResponse {
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
    let (mut sender, mut receiver) = socket.split();

    let topic = "/room1/controller1/cmd";
    let (tx_subscribe, rx_subscribe) = oneshot::channel::<watch::Receiver<Arc<String>>>();
    let (tx_quit, mut rx_quit) = oneshot::channel::<()>();

    let subscribe_task = tokio::spawn(async move {
        debug!("Watcher task started");
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
                            let m = Message::Text(v.to_string());
                            let _ = sender.send(m).await;
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

    while let Some(msg) = receiver.next().await {
        debug!("Received message: {:?}", msg);
    }

    let message = ActorMessage::Subscribe {
        topic: String::from(topic),
        respond_to: tx_subscribe,
    };
    mqtt.send(message).await;

    if tx_quit.send(()).is_ok() {
        subscribe_task.await.unwrap();
    }

    // returning from the handler closes the websocket connection
    debug!("Websocket context {who} destroyed");
}
