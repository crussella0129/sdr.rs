//! Core processing traits for composable SDR block pipelines.

use crate::tag::StreamTag;
use thiserror::Error;

/// Standard SDR errors.
#[derive(Error, Debug)]
pub enum SdrError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Hardware error: {0}")]
    Hardware(String),

    #[error(
        "Buffer overflow: attempted to write {written} samples but only {available} available"
    )]
    BufferOverflow { written: usize, available: usize },

    #[error("Buffer underflow")]
    BufferUnderflow,

    #[error("Format error: {0}")]
    Format(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Protocol error: {0}")]
    Protocol(String),
}

pub type Result<T> = std::result::Result<T, SdrError>;

/// Block processing trait for transforming a stream of input samples into output samples.
pub trait Block<In, Out>: Send + Sync {
    /// Process input slice and write result into output vector or slice.
    /// Returns number of input samples consumed and output samples produced.
    fn process(&mut self, input: &[In], output: &mut Vec<Out>) -> Result<(usize, usize)>;

    /// Reset internal state (e.g. filter delay lines, PLL phase).
    fn reset(&mut self) {}
}

/// A signal source producing sample buffers.
pub trait Source<T>: Send + Sync {
    /// Fill buffer with next batch of samples. Returns number of samples read.
    fn read_samples(&mut self, buffer: &mut [T]) -> Result<usize>;

    /// Retrieve stream tags associated with recent sample batches.
    fn get_tags(&mut self) -> Vec<StreamTag> {
        Vec::new()
    }
}

/// A signal sink consuming sample buffers.
pub trait Sink<T>: Send + Sync {
    /// Write batch of samples into sink. Returns number of samples accepted.
    fn write_samples(&mut self, buffer: &[T]) -> Result<usize>;

    /// Pass stream tags to sink.
    fn put_tags(&mut self, _tags: &[StreamTag]) {}

    /// Flush any remaining buffered samples.
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A simple in-memory linear pipeline runner connecting a source, a processing block, and a sink.
pub struct LinearPipeline<In, Out> {
    source: Box<dyn Source<In>>,
    block: Box<dyn Block<In, Out>>,
    sink: Box<dyn Sink<Out>>,
    /// Preallocated input buffer reused across every `step()` (no per-call allocation).
    in_buf: Vec<In>,
    /// Reused output buffer for the block stage.
    out_buf: Vec<Out>,
}

impl<In: Default + Clone + 'static, Out: Default + Clone + 'static> LinearPipeline<In, Out> {
    pub fn new(
        source: Box<dyn Source<In>>,
        block: Box<dyn Block<In, Out>>,
        sink: Box<dyn Sink<Out>>,
        in_chunk_size: usize,
    ) -> Self {
        let in_chunk_size = in_chunk_size.max(64);
        Self {
            source,
            block,
            sink,
            in_buf: vec![In::default(); in_chunk_size],
            out_buf: Vec::with_capacity(in_chunk_size),
        }
    }

    /// Run one iteration of pipeline processing.
    /// Returns number of samples processed.
    ///
    /// Reuses preallocated input and output buffers rather than allocating on
    /// each call, keeping the hot path allocation-free.
    pub fn step(&mut self) -> Result<usize> {
        let samples_read = self.source.read_samples(&mut self.in_buf)?;
        if samples_read == 0 {
            return Ok(0);
        }

        self.out_buf.clear();
        let (consumed, _produced) = self
            .block
            .process(&self.in_buf[..samples_read], &mut self.out_buf)?;
        if !self.out_buf.is_empty() {
            self.sink.write_samples(&self.out_buf)?;
        }
        Ok(consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Emits the integer sequence `0..max`.
    struct CountSource {
        next: i32,
        max: i32,
    }
    impl Source<i32> for CountSource {
        fn read_samples(&mut self, buffer: &mut [i32]) -> Result<usize> {
            let mut n = 0;
            for slot in buffer.iter_mut() {
                if self.next >= self.max {
                    break;
                }
                *slot = self.next;
                self.next += 1;
                n += 1;
            }
            Ok(n)
        }
    }

    /// Doubles each sample.
    struct DoubleBlock;
    impl Block<i32, i32> for DoubleBlock {
        fn process(&mut self, input: &[i32], output: &mut Vec<i32>) -> Result<(usize, usize)> {
            output.clear();
            output.extend(input.iter().map(|x| x * 2));
            Ok((input.len(), output.len()))
        }
    }

    /// Collects everything written into a shared vector.
    struct CollectSink {
        data: Arc<Mutex<Vec<i32>>>,
    }
    impl Sink<i32> for CollectSink {
        fn write_samples(&mut self, buffer: &[i32]) -> Result<usize> {
            self.data.lock().unwrap().extend_from_slice(buffer);
            Ok(buffer.len())
        }
    }

    fn build_pipeline(chunk: usize) -> (LinearPipeline<i32, i32>, Arc<Mutex<Vec<i32>>>) {
        let collected = Arc::new(Mutex::new(Vec::new()));
        let pipeline = LinearPipeline::new(
            Box::new(CountSource { next: 0, max: 10 }),
            Box::new(DoubleBlock),
            Box::new(CollectSink {
                data: collected.clone(),
            }),
            chunk,
        );
        (pipeline, collected)
    }

    #[test]
    fn test_linear_pipeline_multi_step_correctness() {
        let (mut pipeline, collected) = build_pipeline(4);
        // Drive to completion across multiple steps.
        let mut guard = 0;
        while pipeline.step().unwrap() > 0 {
            guard += 1;
            assert!(guard < 100, "pipeline did not terminate");
        }
        let expected: Vec<i32> = (0..10).map(|x| x * 2).collect();
        assert_eq!(*collected.lock().unwrap(), expected);
    }

    #[test]
    fn test_linear_pipeline_buffer_reused() {
        let (mut pipeline, _collected) = build_pipeline(8);
        // The input buffer is preallocated to the (min-64-clamped) chunk size.
        assert!(pipeline.in_buf.capacity() >= 64);
        let ptr_before = pipeline.in_buf.as_ptr();
        for _ in 0..3 {
            let _ = pipeline.step().unwrap();
        }
        // Same backing allocation after several steps: no per-call reallocation.
        assert_eq!(pipeline.in_buf.as_ptr(), ptr_before);
        assert!(pipeline.in_buf.capacity() >= 64);
    }
}
