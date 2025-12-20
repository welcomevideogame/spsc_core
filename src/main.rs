use std::{error::Error, time::Duration};

use futures::StreamExt;
use tokio::join;

use crate::spsc::{Receiver, ReceiverStream, Sender, channel};

mod blocking_spsc;
mod spsc;

async fn send_loop(tx: Sender<i32>) -> Result<(), Box<dyn Error + Send>> {
    for i in 0..10 {
        tx.send(i).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    println!("Done with send");
    Ok(())
}

async fn recv_loop(rx: Receiver<i32>) -> Option<()> {
    let mut x = ReceiverStream::new(&rx);
    while let Some(x) = x.next().await {
        println!("Got {:?}", x);
    }
    println!("Done with recv");
    None
}

#[tokio::main]
async fn main() {
    let (tx, rx) = channel::<i32>(10);
    let (_, _) = join!(tokio::spawn(send_loop(tx)), tokio::spawn(recv_loop(rx)));
}
