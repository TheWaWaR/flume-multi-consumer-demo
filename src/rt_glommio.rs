use std::time::Duration;

use flume::{bounded, Sender};
use glommio::{CpuSet, LocalExecutorPoolBuilder, PoolPlacement};

use super::flume_recv;

pub fn run_glommio() {
    let cpu_set = CpuSet::online().expect("online cpus");
    let cpu_num = num_cpus::get();
    let placement = PoolPlacement::MaxSpread(cpu_num, Some(cpu_set));
    let (sender, receiver) = bounded::<(usize, usize)>(2);
    LocalExecutorPoolBuilder::new(placement)
        .on_all_shards(|| async move {
            let id = glommio::executor().id();
            println!("Starting glommio runtime...");
            let receiver = receiver.clone();
            glommio::spawn_local(flume_recv(id, receiver)).detach();
            let sender = sender.clone();
            flume_send_glommio(id, sender).await;
        })
        .unwrap()
        .join_all();
}

pub async fn flume_send_glommio(id: usize, sender: Sender<(usize, usize)>) {
    loop {
        for n in 0usize.. {
            if let Err(err) = sender.send_async((id, n)).await {
                panic!("send error: {:?}", err);
            }
            glommio::timer::sleep(Duration::from_secs(1)).await;
        }
    }
}
