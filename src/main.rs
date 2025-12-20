use std::{error::Error, time::Duration};

use tokio::join;

use crate::spsc::{Receiver, Sender};

mod spsc;

async fn send_loop(tx: Sender<i32>) -> Result<(), Box<dyn Error>> {
    for i in 0..10 {
        println!("Sending value {i}");
        tx.send(i)?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}

async fn recv_loop(rx: Receiver<i32>) -> Result<(), ()> {
    loop {
        dbg!(rx.recv());
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = spsc::channel::<i32>(10);
    let (x, y) = join!(send_loop(tx), recv_loop(rx));
    _ = dbg!(x, y);
}
