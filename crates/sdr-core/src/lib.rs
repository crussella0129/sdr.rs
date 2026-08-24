//! # sdr-core
//!
//! Foundational sample representations, stream tags, lock-free ring buffers,
//! processing traits, and regulatory compliance for `sdr.rs`.

#![forbid(unsafe_code)]

pub mod buffer;
pub mod compliance;
pub mod sample;
pub mod tag;
pub mod traits;

pub use buffer::{RingBuffer, SharedRingBuffer};
pub use compliance::{
    BandType, ComplianceResult, Jurisdiction, ParseJurisdictionError, RegulatoryBand,
    RegulatoryDatabase, TransmissionPlan,
};
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

    #[test]
    fn test_regulatory_compliance_ism_bands() {
        // Test US recommended bands with encryption requirement
        let us_encrypted_bands =
            RegulatoryDatabase::query_recommended_bands(Jurisdiction::US, true);
        assert!(us_encrypted_bands
            .iter()
            .any(|b| b.name.contains("915 MHz ISM")));
        // Must NOT contain 2m or 70cm amateur bands
        assert!(!us_encrypted_bands
            .iter()
            .any(|b| b.band_type == BandType::Amateur));

        // Test EU recommended bands
        let eu_bands = RegulatoryDatabase::query_recommended_bands(Jurisdiction::EU, true);
        assert!(eu_bands.iter().any(|b| b.name.contains("868 MHz SRD")));

        // Test compliant transmission check
        let check_us_915 = RegulatoryDatabase::check_compliance(&TransmissionPlan {
            jurisdiction: Jurisdiction::US,
            center_frequency_hz: 915_000_000.0,
            occupied_bandwidth_hz: 100_000.0,
            eirp_dbm: 20.0, // 20 dBm (100 mW)
            duty_cycle_pct: 100.0,
            encrypted: true, // encrypted SSH payload
        });
        assert!(matches!(check_us_915, ComplianceResult::Compliant { .. }));
    }

    #[test]
    fn test_regulatory_compliance_amateur_encryption_rejection() {
        // Transmitting encrypted payload on 144.2 MHz in the US is strictly illegal
        let check_us_2m_encrypted = RegulatoryDatabase::check_compliance(&TransmissionPlan {
            jurisdiction: Jurisdiction::US,
            center_frequency_hz: 144_200_000.0,
            occupied_bandwidth_hz: 20_000.0,
            eirp_dbm: 10.0,
            duty_cycle_pct: 100.0,
            encrypted: true,
        });
        match check_us_2m_encrypted {
            ComplianceResult::NonCompliant { reasons } => {
                assert!(
                    reasons.iter().any(|r| r.contains("ENCRYPTION PROHIBITED")),
                    "Expected encryption prohibition warning on 2m amateur band"
                );
            }
            ComplianceResult::Compliant { .. } => {
                panic!("Encrypted transmission on 2m Amateur band should NOT be compliant!");
            }
        }
    }
}
