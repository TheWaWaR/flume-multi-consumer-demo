use std::env;
use std::time::Duration;

use flume::{bounded, Receiver, Sender};
use glommio::{LocalExecutorBuilder, Placement};

fn main() {
    let mut args = env::args();
    match args.nth(1).unwrap().as_str() {
        "glommio" => run_glommio(),
        "tokio" => run_tokio(),
        action => panic!("invalid action: {}", action),
    }
}

fn run_glommio() {
    LocalExecutorBuilder::new(Placement::Fixed(0))
        .spawn(|| async move {
            println!("Starting glommio runtime...");
            let (sender, receiver) = bounded::<usize>(2);
            for id in [1, 2, 3] {
                let receiver = receiver.clone();
                glommio::spawn_local(flume_recv(id, receiver)).detach();
            }
            flume_send(sender, true).await;
        })
        .unwrap()
        .join()
        .unwrap();
}

fn run_tokio() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async move {
            println!("Starting tokio runtime...");
            let (sender, receiver) = bounded::<usize>(2);
            for id in [1, 2, 3] {
                let receiver = receiver.clone();
                tokio::spawn(flume_recv(id, receiver));
            }
            flume_send(sender, false).await;
        });
}

async fn flume_send(sender: Sender<usize>, is_glommio: bool) {
    loop {
        for idx in 0.. {
            if let Err(err) = sender.send_async(idx).await {
                panic!("send error: {:?}", err);
            }
            if is_glommio {
                glommio::timer::sleep(Duration::from_secs(1)).await;
            } else {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn flume_recv(id: usize, receiver: Receiver<usize>) {
    loop {
        let value = match receiver.recv_async().await {
            Ok(value) => value,
            Err(err) => panic!("[id:{}] recv error: {:?}", id, err),
        };
        println!("[id:{}] received value: {}", id, value);
    }
}
