mod spsc;

#[tokio::main]
async fn main() {
    let (tx, rx) = spsc::channel::<i32>(10);
}
