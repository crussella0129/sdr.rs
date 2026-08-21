//! In-band stream metadata tags for sample stream synchronization.

use serde::{Deserialize, Serialize};

/// Data value attached to an in-band stream tag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TagValue {
    Float(f64),
    Int(i64),
    String(String),
    Bool(bool),
    Bytes(Vec<u8>),
}

impl From<f64> for TagValue {
    fn from(v: f64) -> Self {
        TagValue::Float(v)
    }
}

impl From<f32> for TagValue {
    fn from(v: f32) -> Self {
        TagValue::Float(v as f64)
    }
}

impl From<i64> for TagValue {
    fn from(v: i64) -> Self {
        TagValue::Int(v)
    }
}

impl From<u64> for TagValue {
    fn from(v: u64) -> Self {
        TagValue::Int(v as i64)
    }
}

impl From<usize> for TagValue {
    fn from(v: usize) -> Self {
        TagValue::Int(v as i64)
    }
}

impl From<String> for TagValue {
    fn from(v: String) -> Self {
        TagValue::String(v)
    }
}

impl From<&str> for TagValue {
    fn from(v: &str) -> Self {
        TagValue::String(v.to_string())
    }
}

impl From<bool> for TagValue {
    fn from(v: bool) -> Self {
        TagValue::Bool(v)
    }
}

/// A metadata tag attached to a sample offset in a continuous IQ stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamTag {
    /// Absolute or relative sample offset where this tag applies.
    pub offset: u64,
    /// Identifier key for the tag (e.g. "rx_freq", "sample_rate", "rx_gain").
    pub key: String,
    /// Value carried by the tag.
    pub value: TagValue,
}

impl StreamTag {
    /// Create a new stream tag.
    pub fn new<K: Into<String>, V: Into<TagValue>>(offset: u64, key: K, value: V) -> Self {
        Self {
            offset,
            key: key.into(),
            value: value.into(),
        }
    }

    /// Helper to create a center frequency tag.
    pub fn frequency(offset: u64, hz: f64) -> Self {
        Self::new(offset, "rx_freq", hz)
    }

    /// Helper to create a sample rate tag.
    pub fn sample_rate(offset: u64, rate_hz: f64) -> Self {
        Self::new(offset, "sample_rate", rate_hz)
    }

    /// Helper to create a gain tag.
    pub fn gain(offset: u64, gain_db: f64) -> Self {
        Self::new(offset, "rx_gain", gain_db)
    }

    /// Helper to create a burst start tag.
    pub fn burst_start(offset: u64) -> Self {
        Self::new(offset, "burst_start", true)
    }

    /// Helper to create a burst end tag.
    pub fn burst_end(offset: u64) -> Self {
        Self::new(offset, "burst_end", true)
    }
}

/// Collection of stream tags with helper methods for range queries.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagQueue {
    tags: Vec<StreamTag>,
}

impl TagQueue {
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    pub fn push(&mut self, tag: StreamTag) {
        self.tags.push(tag);
        self.tags.sort_by_key(|t| t.offset);
    }

    /// Get all tags within the sample range `[start, end)`.
    pub fn get_range(&self, start: u64, end: u64) -> Vec<StreamTag> {
        self.tags
            .iter()
            .filter(|t| t.offset >= start && t.offset < end)
            .cloned()
            .collect()
    }

    /// Remove tags with offset strictly less than `until_offset`.
    pub fn prune_before(&mut self, until_offset: u64) {
        self.tags.retain(|t| t.offset >= until_offset);
    }

    /// Number of queued tags.
    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
}
