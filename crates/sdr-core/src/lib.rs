//! # sdr-core
//!
//! Foundational sample representations, stream tags, lock-free ring buffers,
//! and processing traits for `sdr.rs`.

pub mod buffer;
pub mod sample;
pub mod tag;
pub mod traits;

pub use buffer::{RingBuffer, SharedRingBuffer};
pub use sample::{
    convert_samples, Complex32, Complex64, ComplexI16, ComplexI8, ComplexU8, Sample, SampleFormat,
};
pub use tag::{StreamTag, TagQueue, TagValue};
pub use traits::{Block, LinearPipeline, Result, SdrError, Sink, Source};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_conversions() {
        let s16 = ComplexI16::new(16384, -16384);
        let cf32 = s16.to_complex32();
        assert!((cf32.re - 0.5).abs() < 1e-4);
        assert!((cf32.im + 0.5).abs() < 1e-4);

        let roundtrip = ComplexI16::from_complex32(cf32);
        assert_eq!(roundtrip.re, 16383); // slight rounding is normal
        assert_eq!(roundtrip.im, -16383);
    }

    #[test]
    fn test_ring_buffer_fifo_order() {
        let rb = RingBuffer::<Complex32>::new(16);
        assert!(rb.is_empty());
        assert_eq!(rb.capacity(), 15);

        let input = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(3.0, 4.0),
            Complex32::new(5.0, 6.0),
        ];
        let written = rb.write(&input);
        assert_eq!(written, 3);
        assert_eq!(rb.available_read(), 3);

        let mut output = vec![Complex32::default(); 3];
        let read = rb.read(&mut output);
        assert_eq!(read, 3);
        assert_eq!(output, input);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_ring_buffer_wraparound() {
        let rb = RingBuffer::<i32>::new(8);
        for cycle in 0..10 {
            let data = vec![cycle * 10, cycle * 10 + 1, cycle * 10 + 2];
            assert_eq!(rb.write(&data), 3);
            let mut read_buf = vec![0; 3];
            assert_eq!(rb.read(&mut read_buf), 3);
            assert_eq!(read_buf, data);
        }
    }

    #[test]
    fn test_stream_tag_propagation() {
        let mut queue = TagQueue::new();
        queue.push(StreamTag::frequency(0, 915.0e6));
        queue.push(StreamTag::sample_rate(0, 2.0e6));
        queue.push(StreamTag::gain(1000, 40.0));
        queue.push(StreamTag::burst_start(2000));
        queue.push(StreamTag::burst_end(3000));

        assert_eq!(queue.len(), 5);

        let tags_chunk1 = queue.get_range(0, 1000);
        assert_eq!(tags_chunk1.len(), 2);
        assert_eq!(tags_chunk1[0].key, "rx_freq");
        assert_eq!(tags_chunk1[1].key, "sample_rate");

        let tags_chunk2 = queue.get_range(1000, 2500);
        assert_eq!(tags_chunk2.len(), 2);
        assert_eq!(tags_chunk2[0].key, "rx_gain");
        assert_eq!(tags_chunk2[1].key, "burst_start");

        queue.prune_before(2000);
        assert_eq!(queue.len(), 2);
    }
}
