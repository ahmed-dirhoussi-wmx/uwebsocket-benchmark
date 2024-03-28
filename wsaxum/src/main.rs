use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info};

use std::ops::ControlFlow;

use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use serde_json;

const WRITE_FACTOR: usize = 4;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    pub client_id: usize,
    pub msg_id: usize,
    pub msg: String,
    pub created_at: u128,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerMessage {
    client_id: usize,
    msg_id: usize,
    msg: String,
    created_at: u128,
    client_ts: u128,
    server_latency: u128,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/ws", get(ws_handler));

    // let localhost_v4 = SocketAddr::new(Ipv4Addr::new(127,0,0,1).into(),3000);
    let localhost_v4 = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 3000);
    let listener = TcpListener::bind(&localhost_v4).await.unwrap();

    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    debug!("Client at {addr} connected.");
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, client_addr: SocketAddr) {
    while let Some(Ok(msg)) = socket.recv().await {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        match process_ws_message(msg, client_addr) {
            ControlFlow::Continue(Some(msg)) => {
                let response = serde_json::to_vec(&ServerMessage {
                    client_id: msg.client_id,
                    msg_id: msg.msg_id,
                    msg: msg.msg,
                    created_at: now,
                    client_ts: msg.created_at,
                    server_latency: now - msg.created_at,
                })
                .expect("serialize msg");
                debug!(latency = now - msg.created_at, "latency ");
                for _ in 0..WRITE_FACTOR {
                    if socket
                        .send(Message::Binary(response.clone()))
                        .await
                        .is_err()
                    {
                        error!("client {client_addr} abruptly disconnected");
                        return;
                    }
                }
            }
            ControlFlow::Continue(None) => {
                debug!("client unhandled message");
            }
            ControlFlow::Break(_) => break,
        }
    }
    debug!("handling for {client_addr} done.");
}

fn process_ws_message(
    msg: Message,
    client_addr: SocketAddr,
) -> ControlFlow<(), Option<ClientMessage>> {
    match msg {
        Message::Binary(data) => {
            let client_msg: ClientMessage =
                serde_json::from_slice(&data).expect("deserialize data failed");
            debug!("{} received client_msg {:?}", client_addr, &client_msg);
            return ControlFlow::Continue(Some(client_msg));
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                debug!(
                    "{} sent close with code {} and reason `{}`",
                    client_addr, cf.code, cf.reason
                );
            } else {
                error!("{client_addr} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }
        _ => {
            debug!("{} sent unhandled message: {msg:?}", client_addr);
            ControlFlow::Continue(None)
        }
    }
}
