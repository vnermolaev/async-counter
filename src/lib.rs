use std::future::Future;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// Globally available counter with a defined target.
#[derive(Debug, Clone)]
pub struct Counter {
    value: Arc<AtomicUsize>,
    target: usize,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl Counter {
    const MUST_LOCK: &'static str = "Counter inner mutex must lock";

    /// Create a [Counter] starting at `from` with `target`.
    pub fn new(from: usize, target: usize) -> Self {
        Self {
            value: Arc::new(AtomicUsize::new(from)),
            target,
            waker: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a [Counter] starting at 0 with `target`.
    pub fn to(target: usize) -> Self {
        Self::new(0, target)
    }

    /// Inner function incrementing the [Counter] value and waking a waker if any.
    fn inc(&self, rhs: usize) {
        self.value.fetch_add(rhs, Ordering::SeqCst);
        if let Some(waker) = self.waker.lock().expect(Self::MUST_LOCK).take() {
            waker.wake()
        }
    }

    /// Inner function decrementing the [Counter] value and waking a waker if any.
    fn dec(&self, rhs: usize) {
        self.value.fetch_sub(rhs, Ordering::SeqCst);
        if let Some(waker) = self.waker.lock().expect(Self::MUST_LOCK).take() {
            waker.wake()
        }
    }
}

impl Future for Counter {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let value = self.value.load(Ordering::SeqCst);
        if value >= self.target {
            Poll::Ready(value)
        } else {
            *self.waker.lock().expect(Self::MUST_LOCK) = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl AddAssign<usize> for Counter {
    fn add_assign(&mut self, rhs: usize) {
        self.inc(rhs);
    }
}

impl SubAssign<usize> for Counter {
    fn sub_assign(&mut self, rhs: usize) {
        self.dec(rhs);
    }
}

impl Add<usize> for Counter {
    type Output = Self;

    fn add(mut self, rhs: usize) -> Self::Output {
        self += rhs;
        self
    }
}

impl Sub<usize> for Counter {
    type Output = Self;

    fn sub(mut self, rhs: usize) -> Self::Output {
        self -= rhs;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::Counter;
    use log::debug;
    use std::ops::Mul;
    use std::time::Duration;
    use tokio::time;

    #[tokio::test]
    async fn counter_counts_up() {
        let _ = pretty_env_logger::try_init();

        let counting_interval = Duration::from_millis(10);

        let target = 10;
        let counter = Counter::to(target);
        let mut count = counter.clone();

        // Spawn a task to update the counter.
        tokio::spawn(async move {
            for i in 0u8..20 {
                // Simulate some processing
                time::sleep(counting_interval).await;
                debug!("Tick {i}");
                count = count + 5;
            }
        });

        // Wait for the target to be satisfied.
        let r = time::timeout(counting_interval.mul(20), counter).await;
        assert!(matches!(r, Ok(t) if t == target));

        debug!("Counter target is reached!");
    }

    #[tokio::test]
    async fn counter_counts_up_and_down() {
        let _ = pretty_env_logger::try_init();

        let counting_interval = Duration::from_millis(10);

        let target = 10;
        let counter = Counter::to(target);
        let mut count = counter.clone();

        // Spawn a task to update the counter.
        tokio::spawn(async move {
            for i in 0u8..3 {
                time::sleep(counting_interval).await;
                debug!("Tick {i}");
                count += 5;
            }
            // count = 15, future must have been triggered.

            count -= 6;
            // count = 9.

            count += 3;
            //count = 12, future must have been triggered.
        });

        // Wait for the target to be satisfied.
        let r = time::timeout(counting_interval.mul(20), counter.clone()).await;
        assert!(matches!(r, Ok(t) if t == target));

        // Give the child task time to decrement the counter.
        time::sleep(counting_interval.mul(2)).await;

        // Wait for the target to be satisfied.
        let r = time::timeout(counting_interval.mul(20), counter).await;
        debug!("{r:?}");
        assert!(matches!(r, Ok(t) if t == 12));

        debug!("Counter target is reached!");
    }
}
