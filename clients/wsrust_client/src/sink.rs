use std::path::PathBuf;

use tokio::{fs::OpenOptions, io::BufWriter};
use tracing::info;

use tokio::io::AsyncWriteExt;

pub async fn write_results(
    rx: flume::Receiver<String>,
    n_clients: usize,
    batch_size: usize,
    n_batch: usize,
    wait_ms: usize,
    result_dir: PathBuf,
) {
    let file_name = format!(
        "result_c{}_b{}_n{}_w{}.csv",
        n_clients, batch_size, n_batch, wait_ms,
    );
    let result_name = result_dir.join(file_name);
    info!("Saving results to {:?}", &result_name);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(result_name)
        .await
        .expect("can't open/create file");

    // 10M buffer
    let mut writer = BufWriter::with_capacity(10 * 1024 * 1024, file);

    writer
            .write_all("timestamp,client_id,msg_id,server_created_at,client_created_at,client_latency,server_latency\n".as_bytes())
            .await
            .unwrap();

    while let Ok(msg) = rx.recv_async().await {
        writer.write_all(msg.as_bytes()).await.unwrap();
    }
    writer.flush().await.unwrap()
}
