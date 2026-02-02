# Server

## Configuration

Configuration can be provided through the following mechanism:

If the variable `HCS_ENV_FILE` is set, read that file. Otherwise, try to read the file `.env`.

Environment variables override settings from an env file.

### Environment variables

`HCS_MQTT_CLIENT_ID`: mqtt client id. Must be unique across all clients connected to the same server.

`HCS_MQTT_HOST`: mqtt broker hostname.

`HCS_MQTT_PORT`: mqtt broker port number. Defaults to `1883`.

`HCS_MQTT_USERNAME` and `HCS_MQTT_PASSWORD` credentials to be used to connect to the mqtt broker.

`HCS_MQTT_TRANSPORT` can be set to `tls` to use encryption.

`HCS_MQTT_CACERT_FILE` can be used to provide the ca certificate used to sign the server certificate.

`HCS_MQTT_KEEPALIVE` number of seconds for keep alive packets between mqtt broker and client.

`HCS_PERF_CHANNELBUFSIZE` controls the number of messages that are held in an internal queue. Increase if more concurrent web clients are connected.

`PORT` controls the network port to use for serving the backend.

`RUST_LOG` can be set to `debug`, `info`, `warn` to control the verbosity.

## MQTT Actor Overview

The server exposes a lightweight **actor** that abstracts all MQTT communication. All other parts of the codebase interact with this actor through an `MqttHandle` and a few simple message types. Below is a step‑by‑step guide on how to use it:

### 1. Create the Actor
The actor is started once during server bootstrap via `run_subscriber_actor`. The helper reads all required environment variables (see **Configuration** above) and returns three values:

```rust
let (handle, stop_tx, join_handle) =
    run_subscriber_actor(CHANNEL_BUF_SIZE, mqtt_options_from_env()?).await;
```
* `handle` – an `MqttHandle` used to send commands.
* `stop_tx` – a one‑shot channel that signals the actor to shut down gracefully.
* `join_handle` – the background task’s join handle; await it after sending `stop_tx`.

### 2. Subscribe to a Topic
To receive updates you must **subscribe** first:

```rust
let (sub_responder, sub_receiver) = oneshot::channel();
handle.send(ActorMessage::Subscribe {
    topic: "home/livingroom/light".into(),
    respond_to: sub_responder,
}).await;

// Wait for the actor to give us a watch channel.
let mut watcher_rx = sub_receiver.await.expect("subscribe failed");
```
* The actor registers a `watch::Sender` for that topic and performs an MQTT `SUBSCRIBE` on the broker.
* It returns a `watch::Receiver<Arc<String>>`. Whenever the broker publishes to that topic, the receiver is updated with a JSON string:

```json
{
  "type": "update",
  "topic": "home/livingroom/light",
  "data": "ON"
}
```
* To read values you simply wait for changes:

```rust
while watcher_rx.changed().await.is_ok() {
    let payload = &*watcher_rx.borrow();
    println!("New value: {}", payload);
}
```
The receiver always holds the **latest** message, so you can also read it immediately after subscribing without waiting.

### 3. Publish a Message
Sending data to the broker is straightforward:

```rust
let (pub_responder, pub_receiver) = oneshot::channel();
handle.send(ActorMessage::Publish {
    payload: MqttPayload {
        topic: "home/livingroom/light".into(),
        qos: QoS::AtMostOnce,
        retain: false,
        value: b"OFF".to_vec(),
    },
    respond_to: pub_responder,
}).await;

let result = pub_receiver.await.expect("publish failed");
println!("Publish status: {}", result); // "OK" or "Error"
```
The actor forwards the publish request to `rumqttc::AsyncClient`. The response is sent back through the oneshot channel.

### 4. Graceful Shutdown
When the server stops you should tell the actor to exit and wait for its background task:

```rust
let _ = stop_tx.send(());          // signal shutdown
join_handle.await.expect("actor join failed");
```
The actor will break out of its polling loop, clean up the MQTT connection, and terminate.

### 5. Things to Keep in Mind
* **Topic matching** – The actor only forwards messages that exactly match the subscribed topic string. Wildcards (`+`/`#`) are not automatically expanded.
* **Back‑pressure** – If many commands are queued faster than the actor can process, the `mpsc::Sender` will await until space frees up.
* **Watcher lifetime** – Dropping the `watch::Receiver` removes the watcher from the internal map; subsequent broker messages for that topic will be ignored until a new subscription is made.
* **Error handling** – Publish failures are logged and an `"Error"` string is returned. Subscription errors only log a message but do not panic.

With this pattern you can cleanly integrate MQTT into any async Rust component without dealing with raw sockets or connection state.
