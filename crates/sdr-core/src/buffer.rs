//! High-performance bounded circular ring buffer for SDR sample streams.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A thread-safe lock-free Single-Producer Single-Consumer (SPSC) ring buffer.
pub struct RingBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    mask: usize,
    head: AtomicUsize, // Write index
    tail: AtomicUsize, // Read index
}

impl<T: Default + Clone> RingBuffer<T> {
    /// Create a new ring buffer with at least the requested capacity.
    /// The actual capacity is rounded up to the next power of two.
    pub fn new(capacity: usize) -> Self {
        let actual_cap = capacity.max(2).next_power_of_two();
        let buffer = vec![T::default(); actual_cap];
        Self {
            buffer,
            capacity: actual_cap,
            mask: actual_cap - 1,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Number of elements that can be stored in the buffer.
    pub fn capacity(&self) -> usize {
        self.capacity - 1
    }

    /// Number of elements currently available to read.
    pub fn available_read(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail) & self.mask
    }

    /// Number of empty slots available to write.
    pub fn available_write(&self) -> usize {
        let cap = self.capacity();
        let used = self.available_read();
        cap.saturating_sub(used)
    }

    /// True if no elements are available to read.
    pub fn is_empty(&self) -> bool {
        self.available_read() == 0
    }

    /// True if the buffer is full.
    pub fn is_full(&self) -> bool {
        self.available_write() == 0
    }

    /// Write elements from slice into the ring buffer.
    /// Returns the actual number of elements written.
    pub fn write(&self, data: &[T]) -> usize {
        let avail = self.available_write();
        let to_write = data.len().min(avail);
        if to_write == 0 {
            return 0;
        }

        let head = self.head.load(Ordering::Relaxed);
        let start_idx = head & self.mask;

        // Unsafe block to write to internal buffer without mutex lock (SPSC guarantee)
        let ptr = self.buffer.as_ptr() as *mut T;
        for i in 0..to_write {
            let idx = (start_idx + i) & self.mask;
            unsafe {
                *ptr.add(idx) = data[i].clone();
            }
        }

        self.head
            .store(head.wrapping_add(to_write), Ordering::Release);
        to_write
    }

    /// Read elements from ring buffer into destination slice.
    /// Returns the actual number of elements read.
    pub fn read(&self, dst: &mut [T]) -> usize {
        let avail = self.available_read();
        let to_read = dst.len().min(avail);
        if to_read == 0 {
            return 0;
        }

        let tail = self.tail.load(Ordering::Relaxed);
        let start_idx = tail & self.mask;

        let ptr = self.buffer.as_ptr();
        for i in 0..to_read {
            let idx = (start_idx + i) & self.mask;
            unsafe {
                dst[i] = (*ptr.add(idx)).clone();
            }
        }

        self.tail
            .store(tail.wrapping_add(to_read), Ordering::Release);
        to_read
    }

    /// Clear the ring buffer by resetting read and write pointers.
    pub fn clear(&self) {
        let head = self.head.load(Ordering::Acquire);
        self.tail.store(head, Ordering::Release);
    }
}

unsafe impl<T: Send> Send for RingBuffer<T> {}
unsafe impl<T: Sync> Sync for RingBuffer<T> {}

/// Shared reference-counted RingBuffer handle.
pub type SharedRingBuffer<T> = Arc<RingBuffer<T>>;
