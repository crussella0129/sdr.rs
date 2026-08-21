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

    #[error("Buffer overflow: attempted to write {written} samples but only {available} available")]
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
    in_chunk_size: usize,
}

impl<In: Default + Clone + 'static, Out: Default + Clone + 'static> LinearPipeline<In, Out> {
    pub fn new(
        source: Box<dyn Source<In>>,
        block: Box<dyn Block<In, Out>>,
        sink: Box<dyn Sink<Out>>,
        in_chunk_size: usize,
    ) -> Self {
        Self {
            source,
            block,
            sink,
            in_chunk_size: in_chunk_size.max(64),
        }
    }

    /// Run one iteration of pipeline processing.
    /// Returns number of samples processed.
    pub fn step(&mut self) -> Result<usize> {
        let mut in_buf = vec![In::default(); self.in_chunk_size];
        let samples_read = self.source.read_samples(&mut in_buf)?;
        if samples_read == 0 {
            return Ok(0);
        }

        let mut out_buf = Vec::with_capacity(self.in_chunk_size);
        let (consumed, _produced) = self
            .block
            .process(&in_buf[..samples_read], &mut out_buf)?;
        if !out_buf.is_empty() {
            self.sink.write_samples(&out_buf)?;
        }
        Ok(consumed)
    }
}
