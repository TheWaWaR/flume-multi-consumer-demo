mod rt_glommio;
// mod rt_monoio;
mod rt_tokio;

use std::env;

use flume::Receiver;

use rt_glommio::run_glommio;
// use rt_monoio::run_monoio;
use rt_tokio::run_tokio;

fn main() {
    let mut args = env::args();
    match args.nth(1).unwrap().as_str() {
        // "monoio" => run_monoio(),
        // "glommio-pool" => run_glommio_pool(),
        "glommio" => run_glommio(),
        "tokio" => run_tokio(),
        action => panic!("invalid action: {}", action),
    }
}

async fn flume_recv(id: usize, receiver: Receiver<(usize, usize)>) {
    loop {
        let (sender_id, value) = match receiver.recv_async().await {
            Ok(value) => value,
            Err(err) => panic!("[id:{}] recv error: {:?}", id, err),
        };
        println!(
            "[receiver#{}] received value: {} from {}",
            id, value, sender_id
        );
    }
}
