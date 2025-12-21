# spsc_core

A high-performance Rust library providing Single-Producer Single-Consumer **(SPSC)** queues and other concurrency utilities. Optimized for low-latency, systems-level programming, it offers superior throughput compared to Tokio’s MPSC channels.

> ⚠️ This crate is currently local only. Publishing to crates.io is planned for the future.

## Features
- Lock-free SPSC send/receive operations.
- Async-friendly: `send` and `recv` return `Future`s.
- Minimal overhead with proper task wake-up behavior.
- Supports channel closure with safe drop semantics.
- Safe low-level buffer management using `UnsafeCell` and `MaybeUninit`.

## Installation
Add this to your `Cargo.toml` once published:

```toml
[dependencies]
spsc_lab = "0.1.0"
```

## Example
```rs
use spsc_lab::channel;
use tokio::spawn;

#[tokio::main]
async fn main() {
    let (tx, rx) = channel::<i32>(16);

    spawn(async move { tx.send(42).await.unwrap(); });

    let value = rx.recv().await.unwrap();
    assert_eq!(value, 42);
    println!("Received: {}", value);
}
```

## Performance
Benchmarks show that `spsc_lab` can outperform standard async MPSC implementations such as Tokio’s channels in throughput-sensitive scenarios, making it ideal for performance-critical applications with SPSC requirements.

## Safety
- Internally uses UnsafeCell<MaybeUninit<T>>.
- Only the producer writes to tail slots, and only the consumer reads head slots.
- Requires T: Send for safe cross-task communication.
- Not safe for multiple producers or multiple consumers.
