This crate provides a counter that can be shared among different tasks and can be awaited until it reaches a specified value.

Consider the following scenario.
Several asynchronous tasks are incrementing/decrementing a shared value.
At the same time, another task needs to ensure that the shared value reaches at least a specific target.
The following code demonstrates how to use the `Counter` in a simplified scenario.
One child task increments the shared value, while the main task awaits it to reach a specific target.

```rust 
let counter = Counter::to(10);
let mut count = counter.clone();

// Spawn a task to update the counter.
tokio::spawn(async move { 
    for i in 0u8..20 {
        // Simulate some processing
        time::sleep(Duration::from_secs(1)).await;
        count = count + 5;
        // or
        // count += 5;
    } 
});

// Wait for the target to be satisfied.
counter.await; 
```