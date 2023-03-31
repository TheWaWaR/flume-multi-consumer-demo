use std::env;
use std::time::Duration;

use flume::{bounded, Receiver, Sender};
use glommio::{CpuSet, LocalExecutorBuilder, LocalExecutorPoolBuilder, Placement, PoolPlacement};

fn main() {
    let mut args = env::args();
    match args.nth(1).unwrap().as_str() {
        "glommio-pool" => run_glommio_pool(),
        "glommio" => run_glommio(),
        "tokio" => run_tokio(),
        action => panic!("invalid action: {}", action),
    }
}

fn run_glommio_pool() {
    let cpu_set = CpuSet::online().expect("online cpus");
    let cpu_num = num_cpus::get();
    let placement = PoolPlacement::MaxSpread(cpu_num, Some(cpu_set));
    let (sender, receiver) = bounded::<(usize, usize)>(2);
    LocalExecutorPoolBuilder::new(placement)
        .on_all_shards(|| async move {
            println!("Starting glommio runtime...");
            for id in [1, 2, 3] {
                let receiver = receiver.clone();
                glommio::spawn_local(flume_recv(id, receiver)).detach();
            }
            for id in 1..6 {
                let sender = sender.clone();
                glommio::spawn_local(flume_send_glommio(id, sender)).detach();
            }
            glommio::timer::sleep(Duration::from_secs(100000)).await;
        })
        .unwrap()
        .join_all();
}

fn run_glommio() {
    LocalExecutorBuilder::new(Placement::Fixed(0))
        .spawn(|| async move {
            println!("Starting glommio runtime...");
            let (sender, receiver) = bounded::<(usize, usize)>(2);
            for id in [1, 2, 3] {
                let receiver = receiver.clone();
                glommio::spawn_local(flume_recv(id, receiver)).detach();
            }
            for id in 1..6 {
                let sender = sender.clone();
                glommio::spawn_local(flume_send_glommio(id, sender)).detach();
            }
            glommio::timer::sleep(Duration::from_secs(100000)).await;
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

async fn flume_send_glommio(id: usize, sender: Sender<(usize, usize)>) {
    loop {
        for n in 0usize.. {
            if let Err(err) = sender.send_async((id, n)).await {
                panic!("send error: {:?}", err);
            }
            glommio::timer::sleep(Duration::from_secs(1)).await;
        }
    }
}
async fn flume_send_tokio(id: usize, sender: Sender<(usize, usize)>) {
    loop {
        for n in 0usize.. {
            if let Err(err) = sender.send_async((id, n)).await {
                panic!("send error: {:?}", err);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
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
