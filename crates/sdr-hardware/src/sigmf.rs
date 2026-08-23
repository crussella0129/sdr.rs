//! SigMF (Signal Metadata Format v1.0.0) standard reader and writer.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sdr_core::sample::Complex32;
use sdr_core::traits::{Result, SdrError};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

/// SigMF Global Metadata object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigMfGlobal {
    #[serde(rename = "core:datatype")]
    pub datatype: String,
    #[serde(rename = "core:sample_rate")]
    pub sample_rate: f64,
    #[serde(rename = "core:version")]
    pub version: String,
    #[serde(rename = "core:recorder", skip_serializing_if = "Option::is_none")]
    pub recorder: Option<String>,
    #[serde(rename = "core:description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// SigMF Capture segment metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigMfCapture {
    #[serde(rename = "core:sample_start")]
    pub sample_start: u64,
    #[serde(rename = "core:frequency")]
    pub frequency: f64,
    #[serde(rename = "core:datetime", skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,
}

/// SigMF Annotation segment metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigMfAnnotation {
    #[serde(rename = "core:sample_start")]
    pub sample_start: u64,
    #[serde(rename = "core:sample_count")]
    pub sample_count: u64,
    #[serde(rename = "core:label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "core:comment", skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Top-level SigMF metadata container (`.sigmf-meta`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigMfMetadata {
    pub global: SigMfGlobal,
    #[serde(default)]
    pub captures: Vec<SigMfCapture>,
    #[serde(default)]
    pub annotations: Vec<SigMfAnnotation>,
}

impl SigMfMetadata {
    pub fn new_cf32(sample_rate: f64, center_freq: f64) -> Self {
        Self {
            global: SigMfGlobal {
                datatype: "cf32_le".to_string(),
                sample_rate,
                version: "1.0.0".to_string(),
                recorder: Some("sdr.rs".to_string()),
                description: Some("Recorded with sdr.rs suite".to_string()),
            },
            captures: vec![SigMfCapture {
                sample_start: 0,
                frequency: center_freq,
                datetime: None,
            }],
            annotations: Vec::new(),
        }
    }
}

/// Writer for saving SigMF archive pairs (`.sigmf-meta` and `.sigmf-data`).
pub struct SigMfWriter {
    meta_path: PathBuf,
    _data_path: PathBuf,
    metadata: SigMfMetadata,
    data_writer: BufWriter<File>,
    samples_written: u64,
}

impl SigMfWriter {
    pub fn create<P: AsRef<Path>>(
        base_path: P,
        sample_rate: f64,
        center_freq: f64,
    ) -> Result<Self> {
        let p = base_path.as_ref();
        let meta_path = p.with_extension("sigmf-meta");
        let data_path = p.with_extension("sigmf-data");

        let metadata = SigMfMetadata::new_cf32(sample_rate, center_freq);
        let data_file = File::create(&data_path)?;
        let data_writer = BufWriter::new(data_file);

        Ok(Self {
            meta_path,
            _data_path: data_path,
            metadata,
            data_writer,
            samples_written: 0,
        })
    }

    /// Write a chunk of complex float samples.
    pub fn write_samples(&mut self, samples: &[Complex32]) -> Result<()> {
        for &s in samples {
            self.data_writer.write_f32::<LittleEndian>(s.re)?;
            self.data_writer.write_f32::<LittleEndian>(s.im)?;
        }
        self.samples_written += samples.len() as u64;
        Ok(())
    }

    /// Finalize dataset and write metadata JSON to disk.
    pub fn close(mut self) -> Result<()> {
        self.data_writer.flush()?;
        let meta_file = File::create(&self.meta_path)?;
        serde_json::to_writer_pretty(meta_file, &self.metadata)
            .map_err(|e| SdrError::Format(format!("Failed to write SigMF metadata: {}", e)))?;
        Ok(())
    }
}

/// Reader for loading SigMF archive pairs (`.sigmf-meta` and `.sigmf-data`).
pub struct SigMfReader {
    pub metadata: SigMfMetadata,
    data_reader: BufReader<File>,
}

impl SigMfReader {
    pub fn open<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let p = base_path.as_ref();
        let meta_path = if p.extension().map_or(false, |ext| ext == "sigmf-meta") {
            p.to_path_buf()
        } else {
            p.with_extension("sigmf-meta")
        };
        let data_path = meta_path.with_extension("sigmf-data");

        let meta_file = File::open(&meta_path)?;
        let metadata: SigMfMetadata = serde_json::from_reader(meta_file)
            .map_err(|e| SdrError::Format(format!("Invalid SigMF JSON: {}", e)))?;

        let data_file = File::open(&data_path)?;
        let data_reader = BufReader::new(data_file);

        Ok(Self {
            metadata,
            data_reader,
        })
    }

    /// Read next batch of samples into buffer.
    pub fn read_samples(&mut self, buffer: &mut [Complex32]) -> Result<usize> {
        let is_cf32 =
            self.metadata.global.datatype == "cf32_le" || self.metadata.global.datatype == "cf32";
        let is_cs16 =
            self.metadata.global.datatype == "cs16_le" || self.metadata.global.datatype == "cs16";

        if !is_cf32 && !is_cs16 {
            return Err(SdrError::Format(format!(
                "Unsupported SigMF datatype: {}",
                self.metadata.global.datatype
            )));
        }

        let mut read_count = 0;
        for s in buffer.iter_mut() {
            if is_cf32 {
                match (
                    self.data_reader.read_f32::<LittleEndian>(),
                    self.data_reader.read_f32::<LittleEndian>(),
                ) {
                    (Ok(re), Ok(im)) => {
                        *s = Complex32::new(re, im);
                        read_count += 1;
                    }
                    _ => break,
                }
            } else if is_cs16 {
                match (
                    self.data_reader.read_i16::<LittleEndian>(),
                    self.data_reader.read_i16::<LittleEndian>(),
                ) {
                    (Ok(re), Ok(im)) => {
                        const SCALE: f32 = 1.0 / 32768.0;
                        *s = Complex32::new(re as f32 * SCALE, im as f32 * SCALE);
                        read_count += 1;
                    }
                    _ => break,
                }
            }
        }
        Ok(read_count)
    }
}
