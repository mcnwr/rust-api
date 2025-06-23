use std::sync::mpsc;
use std::thread;
use std::time::Instant;

pub async fn pub_user() {
    const PRODUCERS: u32 = 10;
    const ITERATION: u32 = 1000000;

    let start_time = Instant::now();

    let (tx, rx) = mpsc::channel();
    let mut thread_handles = Vec::new();

    for i in 0..PRODUCERS {
        let tx_clone = tx.clone();

        let handle: thread::JoinHandle<()> = thread::spawn(move || {
            for j in 0..ITERATION {
                let message = (i, j);
                tx_clone.send(message).unwrap();
            }
        });

        thread_handles.push(handle);
    }

    drop(tx);

    let mut message_count = 0u64;

    for _received_message in rx {
        message_count += 1;

        if message_count % 1_000_000 == 0 {
            println!("[Consumer] received total {} messages", message_count);
        }
    }

    let duratiom = start_time.elapsed();

    println!("==========");
    println!("TOTAL: {}", PRODUCERS * ITERATION);
    println!("TIME: {:.?}", duratiom);

    for handle in thread_handles {
        handle.join().unwrap();
    }
}
