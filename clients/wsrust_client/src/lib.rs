pub mod client;
pub mod sink;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

const MSG_SIZE: usize = 1024; // 1 Kb msg
const MSG_BYTES: usize = std::mem::size_of::<ClientMessage>();

#[derive(Serialize, Debug)]
pub struct ClientMessage {
    pub client_id: usize,
    pub msg_id: usize,
    pub msg: String,
    pub created_at: u128,
}

impl ClientMessage {
    fn new(ts: u128, client_id: usize, msg_id: usize) -> Self {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(MSG_SIZE - (MSG_BYTES - std::mem::size_of::<String>()))
            .map(char::from)
            .collect();
        Self {
            client_id,
            msg_id,
            msg: rand_string,
            created_at: ts,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ServerMessage {
    pub client_id: usize,
    pub msg_id: usize,
    pub msg: String,
    // time in ms
    pub created_at: u128,
    pub client_ts: u128,
    pub server_latency: u128,
}

impl ServerMessage {
    pub fn format(&self, recv_ts: u128) -> String {
        format!(
            // timestamp, client_id, msg_id, server_created_at, client_created_at, client_latency, server_latency
            "{},{},{},{},{},{},{}\n",
            recv_ts,
            self.client_id,
            self.msg_id,
            self.created_at,
            self.client_ts,
            recv_ts - self.created_at,
            self.server_latency
        )
    }
}
