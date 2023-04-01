use std::time::Duration;

use flume::{bounded, Sender};

pub fn run_monoio() {
    let (sender, receiver) = bounded::<(usize, usize)>(2);
    let cpu_num = num_cpus::get();
    for id in 0..cpu_num {
        let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
            .enable_timer()
            .build()
            .unwrap();
        rt.block_on(async {
            println!("Starting monoio runtime#{}...", id);
        });
    }
}

pub async fn flume_send_monoio(id: usize, sender: Sender<(usize, usize)>) {
    loop {
        for n in 0usize.. {
            if let Err(err) = sender.send_async((id, n)).await {
                panic!("send error: {:?}", err);
            }
            monoio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
