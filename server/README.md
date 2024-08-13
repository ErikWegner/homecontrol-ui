# Server

^

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

`HCS_PERF_CHANNELBUFSIZE` controls the number of messages that are held in an internal queue. Increase if more
concurrent web clients are connected.

`PORT` controls the network port to use for serving the backend.

`RUST_LOG` can be set to `debug`, `info`, `warn` to control the verbosity.
