This crate implements a counter that can be distributed to different tasks and also can be awaited to reach any given value.

Consider the following example: 
A bunch of async tasks increment/decrement a shared value. Simultaneously, another tasks needs to ensure that
the shared value is at least as large a specific target. The following snippet showcases an application of `Counter`
with a single child task incrementing the shared value, while the main tasks await on it to reach a specific target.
```rust 
let counter = Counter::to(10);
let mut count = counter.clone();

// Spawn a task to update the counter.
tokio::spawn(async move { 
    for i in 0u8..20 {
        // Simulate some processing
        time::sleep(counting_interval).await;
        count = count + 5;
    } 
});

// Wait for the target to be satisfied.
counter.await; 
```