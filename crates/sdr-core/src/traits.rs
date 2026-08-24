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

    /// Map absolute input-stream tags onto absolute output-stream offsets.
    ///
    /// The default mapping is valid only when this invocation consumes and
    /// produces the same number of samples. A rate-changing block that receives
    /// tags must override this method with its exact mapping rather than let the
    /// pipeline guess.
    fn map_tags(
        &mut self,
        tags: &[StreamTag],
        input_start: u64,
        consumed: usize,
        output_start: u64,
        produced: usize,
    ) -> Result<Vec<StreamTag>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }
        if consumed != produced {
            return Err(SdrError::Config(
                "rate-changing block received tags without an explicit tag mapper".to_string(),
            ));
        }

        let consumed = u64::try_from(consumed)
            .map_err(|_| SdrError::Protocol("consumed sample count exceeds u64".to_string()))?;
        let mut mapped = Vec::with_capacity(tags.len());
        for tag in tags {
            let relative = tag.offset.checked_sub(input_start).ok_or_else(|| {
                SdrError::Protocol(format!(
                    "tag offset {} precedes processed input offset {input_start}",
                    tag.offset
                ))
            })?;
            if relative >= consumed {
                return Err(SdrError::Protocol(format!(
                    "tag offset {} is outside the consumed input range",
                    tag.offset
                )));
            }
            let mut tag = tag.clone();
            tag.offset = output_start.checked_add(relative).ok_or_else(|| {
                SdrError::Protocol("mapped output tag offset overflowed u64".to_string())
            })?;
            mapped.push(tag);
        }
        Ok(mapped)
    }

    /// Reset internal state (e.g. filter delay lines, PLL phase).
    fn reset(&mut self) {}
}

/// A signal source producing sample buffers.
pub trait Source<T>: Send + Sync {
    /// Fill buffer with next batch of samples. Returns number of samples read.
    fn read_samples(&mut self, buffer: &mut [T]) -> Result<usize>;

    /// Retrieve tags associated with the most recently read sample batch.
    ///
    /// Tag offsets are absolute positions in the input stream.
    fn get_tags(&mut self) -> Vec<StreamTag> {
        Vec::new()
    }
}

/// A signal sink consuming sample buffers.
pub trait Sink<T>: Send + Sync {
    /// Write batch of samples into sink. Returns number of samples accepted.
    fn write_samples(&mut self, buffer: &[T]) -> Result<usize>;

    /// Pass stream tags to the sink using absolute output-stream offsets.
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
    /// Pending input occupies `in_start..in_end`.
    in_start: usize,
    in_end: usize,
    /// Absolute input offset represented by `in_buf[0]`.
    in_base_offset: u64,
    /// Absolute offset where the next source batch begins.
    next_input_offset: u64,
    /// Absolute tags returned with the current input batch.
    in_tags: Vec<StreamTag>,
    /// Reused output buffer for the block stage.
    out_buf: Vec<Out>,
    /// First output sample not yet accepted by the sink.
    out_start: usize,
    /// True only after a block result has passed validation and been committed.
    out_pending: bool,
    /// Absolute offset where the next accepted output sample belongs.
    next_output_offset: u64,
    /// Mapped tags waiting for their associated output samples to be accepted.
    out_tags: Vec<StreamTag>,
    source_eof: bool,
    sink_flushed: bool,
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
            in_start: 0,
            in_end: 0,
            in_base_offset: 0,
            next_input_offset: 0,
            in_tags: Vec::new(),
            out_buf: Vec::with_capacity(in_chunk_size),
            out_start: 0,
            out_pending: false,
            next_output_offset: 0,
            out_tags: Vec::new(),
            source_eof: false,
            sink_flushed: false,
        }
    }

    /// Run one iteration of pipeline processing.
    /// Returns a positive amount of input or retained-output progress. Zero is
    /// returned only after terminal EOF, all pending work, and one successful
    /// sink flush.
    ///
    /// Reuses preallocated input and output buffers rather than allocating on
    /// each call, keeping the hot path allocation-free.
    pub fn step(&mut self) -> Result<usize> {
        if self.has_pending_output() {
            return self.drain_pending_output();
        }

        if self.source_eof {
            return self.finish_eof();
        }

        if !self.has_pending_input() {
            let samples_read = self.source.read_samples(&mut self.in_buf)?;
            if samples_read > self.in_buf.len() {
                return Err(SdrError::Protocol(format!(
                    "source reported {samples_read} samples for a {}-sample buffer",
                    self.in_buf.len()
                )));
            }
            if samples_read == 0 {
                self.source_eof = true;
                return self.finish_eof();
            }

            self.in_start = 0;
            self.in_end = samples_read;
            self.in_base_offset = self.next_input_offset;
            self.next_input_offset =
                add_offset(self.next_input_offset, samples_read, "input stream offset")?;
            self.in_tags = self.source.get_tags();
            self.in_tags.sort_by_key(|tag| tag.offset);
        }

        let pending_input = self.in_end - self.in_start;
        let input_start = add_offset(self.in_base_offset, self.in_start, "processed input offset")?;
        self.out_buf.clear();
        let (consumed, produced) = self
            .block
            .process(&self.in_buf[self.in_start..self.in_end], &mut self.out_buf)?;

        if consumed > pending_input {
            return Err(SdrError::Protocol(format!(
                "block consumed {consumed} samples from {pending_input} available"
            )));
        }
        if produced > self.out_buf.len() {
            return Err(SdrError::Protocol(format!(
                "block reported {produced} produced samples but output contains {}",
                self.out_buf.len()
            )));
        }
        if produced != self.out_buf.len() {
            return Err(SdrError::Protocol(format!(
                "block output contains {} samples but reported {produced}",
                self.out_buf.len()
            )));
        }
        if consumed == 0 {
            return Err(SdrError::Protocol(
                "block made zero input progress while data is pending".to_string(),
            ));
        }

        let input_end = add_offset(input_start, consumed, "consumed input range")?;
        let tags: Vec<_> = self
            .in_tags
            .iter()
            .filter(|tag| tag.offset >= input_start && tag.offset < input_end)
            .cloned()
            .collect();
        let mut mapped_tags = self.block.map_tags(
            &tags,
            input_start,
            consumed,
            self.next_output_offset,
            produced,
        )?;
        let output_end = add_offset(self.next_output_offset, produced, "produced output range")?;
        if mapped_tags
            .iter()
            .any(|tag| tag.offset < self.next_output_offset || tag.offset >= output_end)
        {
            return Err(SdrError::Protocol(
                "block mapped a tag outside its produced output range".to_string(),
            ));
        }
        mapped_tags.sort_by_key(|tag| tag.offset);

        self.in_tags
            .retain(|tag| tag.offset < input_start || tag.offset >= input_end);
        self.in_start += consumed;
        if self.in_start == self.in_end {
            self.in_start = 0;
            self.in_end = 0;
            self.in_tags.clear();
        }

        self.out_start = 0;
        self.out_tags = mapped_tags;
        self.out_pending = produced > 0;
        if produced > 0 {
            self.drain_pending_output()?;
        }

        Ok(consumed)
    }

    fn has_pending_input(&self) -> bool {
        self.in_start < self.in_end
    }

    fn has_pending_output(&self) -> bool {
        self.out_pending
    }

    fn drain_pending_output(&mut self) -> Result<usize> {
        let pending = &self.out_buf[self.out_start..];
        let accepted = self.sink.write_samples(pending)?;
        if accepted > pending.len() {
            return Err(SdrError::Protocol(format!(
                "sink accepted {accepted} samples from {} offered",
                pending.len()
            )));
        }
        if accepted == 0 {
            return Err(SdrError::Protocol(
                "sink made zero progress while output is pending".to_string(),
            ));
        }

        let accepted_end = add_offset(self.next_output_offset, accepted, "accepted output range")?;
        let tag_count = self
            .out_tags
            .iter()
            .take_while(|tag| tag.offset < accepted_end)
            .count();
        if tag_count > 0 {
            self.sink.put_tags(&self.out_tags[..tag_count]);
            self.out_tags.drain(..tag_count);
        }

        self.out_start += accepted;
        self.next_output_offset = accepted_end;
        if self.out_start == self.out_buf.len() {
            self.out_buf.clear();
            self.out_start = 0;
            self.out_pending = false;
            debug_assert!(self.out_tags.is_empty());
        }
        Ok(accepted)
    }

    fn finish_eof(&mut self) -> Result<usize> {
        if !self.sink_flushed {
            self.sink.flush()?;
            self.sink_flushed = true;
        }
        Ok(0)
    }
}

fn add_offset(offset: u64, count: usize, context: &str) -> Result<u64> {
    let count =
        u64::try_from(count).map_err(|_| SdrError::Protocol(format!("{context} exceeds u64")))?;
    offset
        .checked_add(count)
        .ok_or_else(|| SdrError::Protocol(format!("{context} overflowed u64")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
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

    struct ScriptedSource {
        chunks: VecDeque<(Vec<i32>, Vec<StreamTag>)>,
        current_tags: Vec<StreamTag>,
        reads: Arc<AtomicUsize>,
    }

    impl ScriptedSource {
        fn new(chunks: Vec<(Vec<i32>, Vec<StreamTag>)>) -> (Self, Arc<AtomicUsize>) {
            let reads = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    chunks: chunks.into(),
                    current_tags: Vec::new(),
                    reads: Arc::clone(&reads),
                },
                reads,
            )
        }
    }

    impl Source<i32> for ScriptedSource {
        fn read_samples(&mut self, buffer: &mut [i32]) -> Result<usize> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            let Some((samples, tags)) = self.chunks.pop_front() else {
                self.current_tags.clear();
                return Ok(0);
            };
            assert!(samples.len() <= buffer.len());
            buffer[..samples.len()].copy_from_slice(&samples);
            self.current_tags = tags;
            Ok(samples.len())
        }

        fn get_tags(&mut self) -> Vec<StreamTag> {
            std::mem::take(&mut self.current_tags)
        }
    }

    struct IdentityBlock {
        chunk_sizes: VecDeque<usize>,
        calls: Arc<AtomicUsize>,
    }

    impl IdentityBlock {
        fn new(chunk_sizes: impl Into<VecDeque<usize>>) -> (Self, Arc<AtomicUsize>) {
            let calls = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    chunk_sizes: chunk_sizes.into(),
                    calls: Arc::clone(&calls),
                },
                calls,
            )
        }
    }

    impl Block<i32, i32> for IdentityBlock {
        fn process(&mut self, input: &[i32], output: &mut Vec<i32>) -> Result<(usize, usize)> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let consumed = self
                .chunk_sizes
                .pop_front()
                .unwrap_or(input.len())
                .min(input.len());
            output.clear();
            output.extend_from_slice(&input[..consumed]);
            Ok((consumed, consumed))
        }
    }

    #[derive(Clone, Copy)]
    enum SinkAction {
        Accept(usize),
        Error,
        Oversized,
    }

    #[derive(Clone)]
    struct SinkProbe {
        samples: Arc<Mutex<Vec<i32>>>,
        tags: Arc<Mutex<Vec<StreamTag>>>,
        flushes: Arc<AtomicUsize>,
    }

    struct ScriptedSink {
        actions: VecDeque<SinkAction>,
        probe: SinkProbe,
    }

    impl ScriptedSink {
        fn new(actions: impl Into<VecDeque<SinkAction>>) -> (Self, SinkProbe) {
            let probe = SinkProbe {
                samples: Arc::new(Mutex::new(Vec::new())),
                tags: Arc::new(Mutex::new(Vec::new())),
                flushes: Arc::new(AtomicUsize::new(0)),
            };
            (
                Self {
                    actions: actions.into(),
                    probe: probe.clone(),
                },
                probe,
            )
        }
    }

    impl Sink<i32> for ScriptedSink {
        fn write_samples(&mut self, buffer: &[i32]) -> Result<usize> {
            match self
                .actions
                .pop_front()
                .unwrap_or(SinkAction::Accept(usize::MAX))
            {
                SinkAction::Accept(limit) => {
                    let accepted = limit.min(buffer.len());
                    self.probe
                        .samples
                        .lock()
                        .unwrap()
                        .extend_from_slice(&buffer[..accepted]);
                    Ok(accepted)
                }
                SinkAction::Error => {
                    Err(SdrError::Io(std::io::Error::other("scripted sink error")))
                }
                SinkAction::Oversized => Ok(buffer.len() + 1),
            }
        }

        fn put_tags(&mut self, tags: &[StreamTag]) {
            self.probe.tags.lock().unwrap().extend_from_slice(tags);
        }

        fn flush(&mut self) -> Result<()> {
            self.probe.flushes.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn run_to_eof(pipeline: &mut LinearPipeline<i32, i32>) {
        for _ in 0..100 {
            if pipeline.step().unwrap() == 0 {
                return;
            }
        }
        panic!("pipeline did not reach EOF");
    }

    #[test]
    fn test_linear_pipeline_partial_block_consumption_preserves_remainder() {
        let (source, reads) = ScriptedSource::new(vec![((0..8).collect(), vec![])]);
        let (block, calls) = IdentityBlock::new(VecDeque::from([3, 2, 3]));
        let (sink, probe) = ScriptedSink::new(VecDeque::new());
        let mut pipeline =
            LinearPipeline::new(Box::new(source), Box::new(block), Box::new(sink), 8);

        run_to_eof(&mut pipeline);

        assert_eq!(*probe.samples.lock().unwrap(), (0..8).collect::<Vec<_>>());
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        assert_eq!(reads.load(Ordering::SeqCst), 2, "one batch plus EOF");
    }

    #[test]
    fn test_linear_pipeline_does_not_read_next_chunk_while_input_pending() {
        let (source, reads) =
            ScriptedSource::new(vec![(vec![0, 1, 2], vec![]), (vec![3, 4], vec![])]);
        let (block, _) = IdentityBlock::new(VecDeque::from([1, 1, 1, 2]));
        let (sink, probe) = ScriptedSink::new(VecDeque::new());
        let mut pipeline =
            LinearPipeline::new(Box::new(source), Box::new(block), Box::new(sink), 8);

        assert_eq!(pipeline.step().unwrap(), 1);
        assert_eq!(pipeline.step().unwrap(), 1);
        assert_eq!(reads.load(Ordering::SeqCst), 1);
        assert_eq!(pipeline.step().unwrap(), 1);
        assert_eq!(reads.load(Ordering::SeqCst), 1);
        assert_eq!(pipeline.step().unwrap(), 2);
        assert_eq!(reads.load(Ordering::SeqCst), 2);
        run_to_eof(&mut pipeline);

        assert_eq!(*probe.samples.lock().unwrap(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_linear_pipeline_partial_sink_writes_are_retried_in_order() {
        let (source, _) = ScriptedSource::new(vec![(vec![0, 1, 2, 3, 4, 5], vec![])]);
        let (block, calls) = IdentityBlock::new(VecDeque::new());
        let (sink, probe) = ScriptedSink::new(VecDeque::from([
            SinkAction::Accept(2),
            SinkAction::Accept(1),
            SinkAction::Accept(usize::MAX),
        ]));
        let mut pipeline =
            LinearPipeline::new(Box::new(source), Box::new(block), Box::new(sink), 8);

        run_to_eof(&mut pipeline);

        assert_eq!(*probe.samples.lock().unwrap(), vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_linear_pipeline_sink_error_retains_unwritten_output() {
        let (source, _) = ScriptedSource::new(vec![(vec![10, 11, 12, 13], vec![])]);
        let (block, calls) = IdentityBlock::new(VecDeque::new());
        let (sink, probe) = ScriptedSink::new(VecDeque::from([
            SinkAction::Accept(1),
            SinkAction::Error,
            SinkAction::Accept(usize::MAX),
        ]));
        let mut pipeline =
            LinearPipeline::new(Box::new(source), Box::new(block), Box::new(sink), 8);

        assert_eq!(pipeline.step().unwrap(), 4);
        assert!(matches!(pipeline.step(), Err(SdrError::Io(_))));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(pipeline.out_buf[pipeline.out_start..], [11, 12, 13]);

        run_to_eof(&mut pipeline);
        assert_eq!(*probe.samples.lock().unwrap(), vec![10, 11, 12, 13]);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[derive(Clone, Copy)]
    enum InvalidBlockProgress {
        ConsumedTooMany,
        ProducedTooMany,
        ZeroConsumption,
    }

    struct InvalidBlock(InvalidBlockProgress);

    impl Block<i32, i32> for InvalidBlock {
        fn process(&mut self, input: &[i32], output: &mut Vec<i32>) -> Result<(usize, usize)> {
            output.clear();
            Ok(match self.0 {
                InvalidBlockProgress::ConsumedTooMany => (input.len() + 1, 0),
                InvalidBlockProgress::ProducedTooMany => (1, 1),
                InvalidBlockProgress::ZeroConsumption => (0, 0),
            })
        }
    }

    struct ErrorAfterOutputBlock {
        fail_once: bool,
    }

    impl Block<i32, i32> for ErrorAfterOutputBlock {
        fn process(&mut self, input: &[i32], output: &mut Vec<i32>) -> Result<(usize, usize)> {
            output.clear();
            output.extend_from_slice(input);
            if self.fail_once {
                self.fail_once = false;
                return Err(SdrError::Config("scripted block error".to_string()));
            }
            Ok((input.len(), output.len()))
        }
    }

    #[test]
    fn test_linear_pipeline_invalid_progress_returns_error() {
        for invalid in [
            InvalidBlockProgress::ConsumedTooMany,
            InvalidBlockProgress::ProducedTooMany,
            InvalidBlockProgress::ZeroConsumption,
        ] {
            let (source, reads) = ScriptedSource::new(vec![(vec![1, 2, 3], vec![])]);
            let (sink, _) = ScriptedSink::new(VecDeque::new());
            let mut pipeline = LinearPipeline::new(
                Box::new(source),
                Box::new(InvalidBlock(invalid)),
                Box::new(sink),
                8,
            );

            assert!(matches!(pipeline.step(), Err(SdrError::Protocol(_))));
            assert_eq!(pipeline.in_start, 0);
            assert_eq!(pipeline.in_end, 3);
            assert_eq!(reads.load(Ordering::SeqCst), 1);
        }

        for action in [SinkAction::Accept(0), SinkAction::Oversized] {
            let (source, _) = ScriptedSource::new(vec![(vec![1, 2, 3], vec![])]);
            let (block, calls) = IdentityBlock::new(VecDeque::new());
            let (sink, probe) = ScriptedSink::new(VecDeque::from([action]));
            let mut pipeline =
                LinearPipeline::new(Box::new(source), Box::new(block), Box::new(sink), 8);

            assert!(matches!(pipeline.step(), Err(SdrError::Protocol(_))));
            assert_eq!(pipeline.out_start, 0);
            assert_eq!(pipeline.out_buf, vec![1, 2, 3]);
            assert_eq!(calls.load(Ordering::SeqCst), 1);

            run_to_eof(&mut pipeline);
            assert_eq!(*probe.samples.lock().unwrap(), vec![1, 2, 3]);
            assert_eq!(calls.load(Ordering::SeqCst), 1);
        }

        let (source, _) = ScriptedSource::new(vec![(vec![4, 5, 6], vec![])]);
        let (sink, probe) = ScriptedSink::new(VecDeque::new());
        let mut pipeline = LinearPipeline::new(
            Box::new(source),
            Box::new(ErrorAfterOutputBlock { fail_once: true }),
            Box::new(sink),
            8,
        );
        assert!(matches!(pipeline.step(), Err(SdrError::Config(_))));
        assert!(!pipeline.out_pending);
        assert_eq!(pipeline.in_start, 0);
        run_to_eof(&mut pipeline);
        assert_eq!(*probe.samples.lock().unwrap(), vec![4, 5, 6]);
    }

    #[test]
    fn test_linear_pipeline_source_tags_forwarded_exactly_once() {
        let expected_tags = vec![
            StreamTag::frequency(0, 915.0e6),
            StreamTag::gain(2, 12.0),
            StreamTag::burst_end(5),
        ];
        let (source, _) = ScriptedSource::new(vec![((0..6).collect(), expected_tags.clone())]);
        let (block, _) = IdentityBlock::new(VecDeque::from([2, 1, 3]));
        let (sink, probe) = ScriptedSink::new(VecDeque::from([
            SinkAction::Accept(1),
            SinkAction::Accept(usize::MAX),
        ]));
        let mut pipeline =
            LinearPipeline::new(Box::new(source), Box::new(block), Box::new(sink), 8);

        run_to_eof(&mut pipeline);

        assert_eq!(*probe.samples.lock().unwrap(), (0..6).collect::<Vec<_>>());
        assert_eq!(*probe.tags.lock().unwrap(), expected_tags);
    }

    struct HalvingBlock;

    impl Block<i32, i32> for HalvingBlock {
        fn process(&mut self, input: &[i32], output: &mut Vec<i32>) -> Result<(usize, usize)> {
            output.clear();
            output.extend(input.iter().step_by(2).copied());
            Ok((input.len(), output.len()))
        }
    }

    struct MappedHalvingBlock;

    impl Block<i32, i32> for MappedHalvingBlock {
        fn process(&mut self, input: &[i32], output: &mut Vec<i32>) -> Result<(usize, usize)> {
            output.clear();
            output.extend(input.iter().step_by(2).copied());
            Ok((input.len(), output.len()))
        }

        fn map_tags(
            &mut self,
            tags: &[StreamTag],
            input_start: u64,
            _consumed: usize,
            output_start: u64,
            _produced: usize,
        ) -> Result<Vec<StreamTag>> {
            tags.iter()
                .map(|tag| {
                    let mut mapped = tag.clone();
                    mapped.offset = output_start + (tag.offset - input_start) / 2;
                    Ok(mapped)
                })
                .collect()
        }
    }

    #[test]
    fn test_linear_pipeline_rate_changing_block_requires_tag_mapper() {
        let tags = vec![StreamTag::burst_start(0), StreamTag::gain(2, 4.0)];
        let (source, _) = ScriptedSource::new(vec![(vec![0, 1, 2, 3], tags.clone())]);
        let (sink, _) = ScriptedSink::new(VecDeque::new());
        let mut unmapped =
            LinearPipeline::new(Box::new(source), Box::new(HalvingBlock), Box::new(sink), 8);
        assert!(matches!(unmapped.step(), Err(SdrError::Config(_))));

        let (source, _) = ScriptedSource::new(vec![(vec![0, 1, 2, 3], tags)]);
        let (sink, probe) = ScriptedSink::new(VecDeque::new());
        let mut mapped = LinearPipeline::new(
            Box::new(source),
            Box::new(MappedHalvingBlock),
            Box::new(sink),
            8,
        );
        run_to_eof(&mut mapped);

        assert_eq!(*probe.samples.lock().unwrap(), vec![0, 2]);
        assert_eq!(
            probe
                .tags
                .lock()
                .unwrap()
                .iter()
                .map(|tag| tag.offset)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
    }

    #[test]
    fn test_linear_pipeline_flushes_sink_once_at_eof() {
        let (source, reads) = ScriptedSource::new(vec![(vec![1, 2], vec![])]);
        let (block, _) = IdentityBlock::new(VecDeque::new());
        let (sink, probe) = ScriptedSink::new(VecDeque::new());
        let mut pipeline =
            LinearPipeline::new(Box::new(source), Box::new(block), Box::new(sink), 8);

        assert_eq!(pipeline.step().unwrap(), 2);
        assert_eq!(pipeline.step().unwrap(), 0);
        assert_eq!(pipeline.step().unwrap(), 0);
        assert_eq!(pipeline.step().unwrap(), 0);

        assert_eq!(probe.flushes.load(Ordering::SeqCst), 1);
        assert_eq!(reads.load(Ordering::SeqCst), 2, "one batch plus EOF");
    }
}
