FROM rust:1.76.0 as builder
WORKDIR /app
COPY ./src/ /app/src
COPY ./Cargo.toml .
COPY ./Cargo.lock  .
RUN cargo install --path . --root /usr/local/cargo/

FROM rust:1.76.0
COPY --from=builder /usr/local/cargo/bin/wsaxum /usr/local/bin/wsaxum
RUN apt-get update && \
    apt-get install -y iproute2 iputils-ping 
WORKDIR /app
COPY ./entrypoint.sh .

CMD [ "bash","entrypoint.sh" ]