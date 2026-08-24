//! High-performance bounded circular ring buffer for SDR sample streams.

use crossbeam_queue::ArrayQueue;
use std::sync::Arc;

/// A thread-safe bounded lock-free ring buffer.
///
/// The queue is safe with multiple producers and consumers even though SDR
/// pipelines commonly use it in a single-producer, single-consumer topology.
pub struct RingBuffer<T> {
    queue: ArrayQueue<T>,
}

impl<T: Default + Clone> RingBuffer<T> {
    /// Create a new ring buffer with at least the requested capacity.
    ///
    /// The usable capacity preserves the original ring-buffer contract:
    /// `next_power_of_two(max(2, capacity)) - 1`.
    pub fn new(capacity: usize) -> Self {
        let usable_capacity = capacity.max(2).next_power_of_two() - 1;
        Self {
            queue: ArrayQueue::new(usable_capacity),
        }
    }

    /// Number of elements that can be stored in the buffer.
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }

    /// Snapshot of the number of elements currently available to read.
    ///
    /// Concurrent producers or consumers may change the value immediately
    /// after this method returns.
    pub fn available_read(&self) -> usize {
        self.queue.len()
    }

    /// Snapshot of the number of empty slots available to write.
    ///
    /// Concurrent producers or consumers may change the value immediately
    /// after this method returns.
    pub fn available_write(&self) -> usize {
        self.capacity().saturating_sub(self.available_read())
    }

    /// True if no elements are available to read.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// True if the buffer is full.
    pub fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    /// Write elements from slice into the ring buffer.
    /// Returns the actual number of elements written.
    pub fn write(&self, data: &[T]) -> usize {
        let mut written = 0;
        for value in data {
            if self.queue.push(value.clone()).is_err() {
                break;
            }
            written += 1;
        }
        written
    }

    /// Read elements from ring buffer into destination slice.
    /// Returns the actual number of elements read.
    pub fn read(&self, dst: &mut [T]) -> usize {
        let mut read = 0;
        for slot in dst {
            let Some(value) = self.queue.pop() else {
                break;
            };
            *slot = value;
            read += 1;
        }
        read
    }

    /// Remove every currently queued element.
    ///
    /// Producers must be quiescent while this method runs. Concurrent
    /// availability queries remain safe but should be treated as snapshots.
    pub fn clear(&self) {
        while self.queue.pop().is_some() {}
    }
}

/// Shared reference-counted [`RingBuffer`] handle.
pub type SharedRingBuffer<T> = Arc<RingBuffer<T>>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::thread;

    #[test]
    fn test_ring_buffer_concurrent_spsc_preserves_order_without_loss() {
        let rb = RingBuffer::<usize>::new(32);
        let capacity = rb.capacity();
        let total = capacity * 20 + 7;

        thread::scope(|scope| {
            let producer = scope.spawn(|| {
                for value in 0..total {
                    while rb.write(&[value]) == 0 {
                        thread::yield_now();
                    }
                }
            });
            let consumer = scope.spawn(|| {
                let mut received = Vec::with_capacity(total);
                let mut slot = [0];
                while received.len() < total {
                    if rb.read(&mut slot) == 1 {
                        received.push(slot[0]);
                    } else {
                        thread::yield_now();
                    }
                }
                received
            });

            producer.join().unwrap();
            let received = consumer.join().unwrap();
            assert_eq!(received, (0..total).collect::<Vec<_>>());
        });

        assert!(rb.is_empty());
    }

    #[test]
    fn test_ring_buffer_capacity_and_partial_io() {
        let rb = RingBuffer::<u8>::new(5);
        assert_eq!(rb.capacity(), 7);
        assert_eq!(rb.available_write(), 7);

        assert_eq!(rb.write(&[0, 1, 2, 3, 4, 5, 6, 7, 8]), 7);
        assert!(rb.is_full());
        assert_eq!(rb.write(&[9]), 0);

        let mut prefix = [0; 3];
        assert_eq!(rb.read(&mut prefix), 3);
        assert_eq!(prefix, [0, 1, 2]);
        assert_eq!(rb.write(&[7, 8, 9, 10]), 3);

        let mut remainder = [u8::MAX; 10];
        assert_eq!(rb.read(&mut remainder), 7);
        assert_eq!(&remainder[..7], &[3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(&remainder[7..], &[u8::MAX; 3]);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_ring_buffer_clear_is_safe_and_reusable() {
        let rb = RingBuffer::<u16>::new(8);
        let capacity = rb.capacity();
        assert_eq!(rb.write(&[1, 2, 3, 4, 5]), 5);

        rb.clear();
        assert!(rb.is_empty());
        assert_eq!(rb.capacity(), capacity);
        assert_eq!(rb.available_write(), capacity);

        let values: Vec<_> = (100..100 + capacity as u16).collect();
        assert_eq!(rb.write(&values), capacity);
        let mut output = vec![0; capacity];
        assert_eq!(rb.read(&mut output), capacity);
        assert_eq!(output, values);
    }

    #[test]
    fn test_ring_buffer_send_sync_for_send_samples() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<RingBuffer<Cell<u32>>>();
    }
}
