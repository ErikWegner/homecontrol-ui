# Step 1: Build Stage
FROM rust:1.80.1-bookworm as builder

# Install the necessary dependencies for OpenSSL
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Erstelle ein neues Verzeichnis für den Quellcode
WORKDIR /usr/src/myapp

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create the Cargo.lock file
RUN mkdir src && echo "fn main() {println!(\"Hello, world!\");}" > src/main.rs && cargo build --release && rm -rf src

# Copy the source code
COPY . .

# Build the application in release mode
RUN cargo build --release

# Step 2: Runtime Stage
FROM debian:bookworm-slim

# Install the necessary libraries
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/local/bin

# Copy the compiled binary from the build stage
COPY --from=builder /usr/src/myapp/target/release/homecontrol-ui-server .

# Make binary executable
RUN chmod +x /usr/local/bin/homecontrol-ui-server

# Set the correct permissions and change the ownership of the binary
RUN chown nobody:nogroup /usr/local/bin/homecontrol-ui-server && chmod +x /usr/local/bin/homecontrol-ui-server

# Switch to the 'nobody' user
USER nobody

# Start the application
CMD ["./homecontrol-ui-server"]

