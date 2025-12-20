mod spsc;

fn main() {
    let (tx, rx) = spsc::Channel::<i32>::new(10);
    tx.send(1);
}
