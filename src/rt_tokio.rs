use std::time::Duration;

use flume::{bounded, Sender};

use super::flume_recv;

pub fn run_tokio() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async move {
            println!("Starting tokio runtime...");
            let (sender, receiver) = bounded::<(usize, usize)>(2);
            for id in [1, 2, 3] {
                let receiver = receiver.clone();
                tokio::spawn(flume_recv(id, receiver));
            }
            for id in 1..6 {
                let sender = sender.clone();
                tokio::spawn(flume_send_tokio(id, sender));
            }
            tokio::time::sleep(Duration::from_secs(1000000)).await;
        });
}

pub async fn flume_send_tokio(id: usize, sender: Sender<(usize, usize)>) {
    loop {
        for n in 0usize.. {
            if let Err(err) = sender.send_async((id, n)).await {
                panic!("send error: {:?}", err);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
