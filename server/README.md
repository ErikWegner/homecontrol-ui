# Server

^

## Configuration

Configuration can be provided through the following mechanism:

If the variable `HCS_ENV_FILE` is set, read that file. Otherwise, try to read the file `.env`.

Environment variables override settings from an env file.

### Environment variables

`HCS_PERF_CHANNELBUFSIZE` controls the number of messages that are held in an internal queue. Increase if more
concurrent web clients are connected.

`PORT` controls the network port to use for serving the backend.

`RUST_LOG` can be set to `debug`, `info`, `warn` to control the verbosity.
