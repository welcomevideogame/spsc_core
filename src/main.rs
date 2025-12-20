use std::{error::Error, time::Duration};

use tokio::join;

use crate::spsc::{Receiver, Sender};

mod spsc;

async fn send_loop(tx: Sender<i32>) -> Result<(), Box<dyn Error>> {
    loop {
        tx.send(10)?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn recv_loop(rx: Receiver<i32>) -> Result<(), ()> {
    loop {
        rx.recv();
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = spsc::channel::<i32>(10);
    let (x, y) = join!(send_loop(tx), recv_loop(rx));
    dbg!(x, y);
}
