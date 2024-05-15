use std::{path::PathBuf, time::Instant};
use tokio::task::JoinSet;
use ws_client::client::{connect_ws, BenchClient};
use ws_client::sink::write_results;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use clap::Parser;

/// Rust websocket benchmarking client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long,default_value_t = String::from("ws://localhost:9001/ws"))]
    server: String,
    #[arg(short, default_value_t = 1_000)]
    clients: usize,
    #[arg(short, default_value_t = 1)]
    batch_size: usize,
    #[arg(short, default_value_t = 1)]
    n_batch: usize,
    #[arg(short, default_value_t = 100)]
    wait: usize,
    #[arg(short, default_value_t = 4)]
    rcv_factor: usize,
    #[arg(long)]
    result_dir: PathBuf,
}

async fn connect_clients(server_addr: String, nclients: usize) -> Vec<BenchClient> {
    let mut clients = Vec::with_capacity(nclients);
    // Connect all clients
    let start_time = Instant::now();
    let mut client_set = JoinSet::new();
    for cid in 0..nclients {
        let server = server_addr.clone();
        client_set.spawn(async move {
            loop {
                match connect_ws(cid, server.clone()).await {
                    Ok(conn) => return conn,
                    Err(e) => error!("can't establish conn : {:?}", e),
                }
            }
        });
    }

    while let Some(res) = client_set.join_next().await {
        clients.push(res.expect("can't establish connection to all clients"));
    }
    let end_time = Instant::now();
    info!(
        "Total time taken {:#?}  to establish conn for {} concurrent clients",
        end_time - start_time,
        nclients
    );
    clients
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    // Wait for all our clients to establish a conn

    // Start bg sink task
    let (tx, rx) = flume::unbounded::<String>();
    let sink_task = tokio::spawn(write_results(
        rx,
        args.clients,
        args.batch_size,
        args.n_batch,
        args.wait,
        args.result_dir,
    ));

    let clients = connect_clients(args.server, args.clients).await;

    // Start sending and receiving messages
    let start_time = Instant::now();

    let mut runs = JoinSet::new();
    for client in clients {
        runs.spawn(client.run(
            args.n_batch,
            args.batch_size,
            args.wait,
            args.rcv_factor,
            tx.clone(),
        ));
    }
    while let Some(res) = runs.join_next().await {
        res.expect("run errored")
    }
    // Wait for all client runs
    let end_time = Instant::now();

    // Wait for sink task to finish
    drop(tx);
    info!("Waiting for sink to finish writing...");
    sink_task.await.unwrap();

    info!(
        "Total time taken {:#?} with {} concurrent clients",
        end_time - start_time,
        args.clients
    );
}
