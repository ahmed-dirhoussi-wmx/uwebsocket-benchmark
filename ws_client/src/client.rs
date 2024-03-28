use flume;
use futures::stream::{iter, SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::borrow::Cow;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::{self, sleep};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use tokio::net::TcpStream;
use tracing::{debug, error};

use tokio_tungstenite::tungstenite::protocol::{frame::coding::CloseCode, CloseFrame, Message};
use tokio_tungstenite::{connect_async_with_config, MaybeTlsStream, WebSocketStream};

use crate::{ClientMessage, ServerMessage};

#[allow(deprecated)]
pub async fn connect_ws(client_id: usize, server: String) -> Result<BenchClient, String> {
    debug!(client_id, "Connecting to server");
    let config = WebSocketConfig {
        max_send_queue: None,
        write_buffer_size: 0, // eagarly write
        max_write_buffer_size: 1000000,
        max_message_size: None,
        max_frame_size: None,
        accept_unmasked_frames: true,
    };
    let (ws_stream, response) = connect_async_with_config(server, Some(config), false)
        .await
        .map_err(|e| format!("error connecting {e:?}"))?;
    debug!(client_id, "Connected client to server with {response:?}");
    Ok(BenchClient {
        client_id,
        ws_stream,
    })
}

fn process_message(client_id: usize, msg: Message, recv_ts: u128) -> Option<String> {
    match msg {
        Message::Binary(data) => {
            let msg: ServerMessage =
                serde_json::from_slice(&data).expect("deserialize data failed");

            debug!(client_id, recv_ts, "received msg from server");
            assert!(
                msg.client_id == client_id,
                "server sent different client_id"
            );
            Some(msg.format(recv_ts))
        }
        Message::Close(_) => {
            debug!("received close frame from server");
            None
        }
        _ => unreachable!("message type unimplemented"),
    }
}

async fn run_sender(
    mut sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    client_id: usize,
    batch_size: usize,
    n_batch: usize,
    wait_ms: usize,
) {
    // Distribute workload of clients to not be at lockstep
    sleep(Duration::from_millis(client_id as u64 % 100)).await;
    let mut interval = time::interval(Duration::from_millis(wait_ms as u64));

    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    for batch_idx in 0..n_batch {
        interval.tick().await;

        debug!(client_id, "sending batch {batch_idx}.");

        let start_batch = SystemTime::now();

        let mut batch_msgs = iter((0..batch_size).map(|msg_idx| {
            let start = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let msg = ClientMessage::new(start, client_id, msg_idx + batch_idx);
            let data = serde_json::to_vec(&msg).expect("serialize message");

            Ok(Message::Binary(data))
        }));

        // send_all flushes automagically
        if let Err(e) = sender.send_all(&mut batch_msgs).await {
            error!("Can't send batch to client : {e:?}")
        }

        let ttw = match start_batch.elapsed() {
            Ok(elapsed) => elapsed.as_millis(),
            Err(e) => {
                // an error occurred!
                error!("Time drift: {e:?}");
                0u128
            }
        };

        debug!(client_id, batch_idx, ttw, "time to write batch in (ms)");
    }
    debug!(client_id, "Sending close to server...");
    if let Err(e) = sender
        .send(Message::Close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: Cow::from("Finished msgs"),
        })))
        .await
    {
        error!("Could not send Close due to {e:?}, probably it is ok?");
    };
}

async fn run_recvr(
    mut receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    tx: flume::Sender<String>,
    client_id: usize,
    n_msgs: usize,
) {
    let mut cnt = 0;
    while cnt < n_msgs {
        if let Some(Ok(msg)) = receiver.next().await {
            let recv_ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("can't error")
                .as_millis();
            if let Some(msg) = process_message(client_id, msg, recv_ts) {
                tx.send_async(msg).await.expect("send msg to sink");
                cnt += 1
            } else {
                break;
            }
        } else {
            error!("failed to get next msg from server");
            break;
        }
    }
    debug!(client_id, "Received all messages!")
}

pub struct BenchClient {
    pub client_id: usize,
    pub ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl BenchClient {
    pub async fn run(
        self,
        n_batch: usize,
        batch_size: usize,
        wait_ms: usize,
        rcv_factor: usize,
        tx: flume::Sender<String>,
    ) {
        let (sender, receiver) = self.ws_stream.split();
        let sender_task = tokio::spawn(run_sender(
            sender,
            self.client_id,
            batch_size,
            n_batch,
            wait_ms,
        ));
        let rcvr_task = tokio::spawn(run_recvr(
            receiver,
            tx,
            self.client_id,
            batch_size * n_batch * rcv_factor,
        ));
        let (send_res, recv_res) = tokio::join!(sender_task, rcvr_task);

        if send_res.is_err() | recv_res.is_err() {
            error!("error occured")
        }
        // Waiting
        debug!(self.client_id, "finished without error.")
    }
}
