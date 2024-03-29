FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
WORKDIR /app
COPY ./src/ /app/src
COPY ./Cargo.toml .
COPY ./Cargo.lock  .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./src/ /app/src
COPY ./Cargo.toml .
COPY ./Cargo.lock  .
RUN cargo build --release --bin ws_client 

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/ws_client /usr/local/bin/ws_client
RUN apt-get update && \
    apt-get install -y iproute2 iputils-ping 
COPY ./entrypoint.sh .

CMD [ "bash","entrypoint.sh" ]