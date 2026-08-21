//! RIFF WAV IQ format reader and writer.

use hound::{WavReader, WavSpec, WavWriter};
use sdr_core::sample::Complex32;
use sdr_core::traits::{Result, SdrError};
use std::path::Path;

/// Write complex samples to a 16-bit stereo WAV file (Channel 0 = I, Channel 1 = Q).
pub fn write_iq_wav<P: AsRef<Path>>(
    path: P,
    sample_rate: u32,
    samples: &[Complex32],
) -> Result<()> {
    let spec = WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)
        .map_err(|e| SdrError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    for &s in samples {
        let i_val = (s.re * 32767.0).clamp(-32768.0, 32767.0) as i16;
        let q_val = (s.im * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer
            .write_sample(i_val)
            .map_err(|e| SdrError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        writer
            .write_sample(q_val)
            .map_err(|e| SdrError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    }
    writer
        .finalize()
        .map_err(|e| SdrError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(())
}

/// Read complex samples from a stereo WAV file (Channel 0 = I, Channel 1 = Q).
pub fn read_iq_wav<P: AsRef<Path>>(path: P) -> Result<(u32, Vec<Complex32>)> {
    let mut reader = WavReader::open(path)
        .map_err(|e| SdrError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    let spec = reader.spec();
    if spec.channels != 2 {
        return Err(SdrError::Format(format!(
            "Expected 2 channels for IQ WAV, found {}",
            spec.channels
        )));
    }

    let mut samples = Vec::new();
    if spec.sample_format == hound::SampleFormat::Int && spec.bits_per_sample == 16 {
        let mut iter = reader.samples::<i16>();
        while let (Some(Ok(i_val)), Some(Ok(q_val))) = (iter.next(), iter.next()) {
            const SCALE: f32 = 1.0 / 32768.0;
            samples.push(Complex32::new(i_val as f32 * SCALE, q_val as f32 * SCALE));
        }
    } else if spec.sample_format == hound::SampleFormat::Float {
        let mut iter = reader.samples::<f32>();
        while let (Some(Ok(i_val)), Some(Ok(q_val))) = (iter.next(), iter.next()) {
            samples.push(Complex32::new(i_val, q_val));
        }
    } else {
        return Err(SdrError::Format(format!(
            "Unsupported WAV format: bits={}, format={:?}",
            spec.bits_per_sample, spec.sample_format
        )));
    }

    Ok((spec.sample_rate, samples))
}
