use std::{error::Error, time::Duration};

use tokio::join;

use crate::spsc::{Receiver, Sender};

mod spsc;

async fn send_loop(tx: Sender<i32>) -> Result<(), Box<dyn Error + Send>> {
    for i in 0..10 {
        println!("SendLoop: Send {i}");
        tx.send(i)?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("Sent");
    }
    Ok(())
}

async fn recv_loop(rx: Receiver<i32>) -> Option<()> {
    loop {
        let x = rx.recv()?;
        println!("RecvLoop: Got {x}");
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("Received");
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = spsc::channel::<i32>(10);
    let (x, y) = join!(tokio::spawn(send_loop(tx)), tokio::spawn(recv_loop(rx)));
    _ = dbg!(x, y);
}
