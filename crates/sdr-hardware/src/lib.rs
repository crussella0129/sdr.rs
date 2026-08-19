//! # sdr-hardware
//!
//! Cross-platform SDR hardware abstraction layer, PlutoSDR IIO client, SigMF dataset reader/writer,
//! WAV audio/IQ storage, and mock drivers for `sdr.rs`.

pub mod driver;
pub mod mock;
pub mod pluto;
pub mod sigmf;
pub mod wav;

pub use driver::{DeviceInfo, GainMode, SdrDriver};
pub use mock::{MockSdr, MockSignal};
pub use pluto::{parse_iio_uri, IioTransport, PlutoSdr};
pub use sigmf::{SigMfAnnotation, SigMfCapture, SigMfGlobal, SigMfMetadata, SigMfReader, SigMfWriter};
pub use wav::{read_iq_wav, write_iq_wav};

#[cfg(test)]
mod tests {
    use super::*;
    use sdr_core::sample::Complex32;

    #[test]
    fn test_sdr_driver_trait_mock_streaming() {
        let mut sdr = MockSdr::new(1000000.0, 915.0e6);
        assert_eq!(sdr.name(), "Mock SDR Driver");
        assert!(!sdr.is_active());

        sdr.set_frequency(0, 433.92e6).unwrap();
        sdr.set_gain(0, 35.0).unwrap();
        sdr.start_rx().unwrap();
        assert!(sdr.is_active());

        let mut buf = vec![Complex32::default(); 512];
        let n = sdr.read_samples(&mut buf).unwrap();
        assert_eq!(n, 512);

        // Verify non-zero samples generated
        let energy: f32 = buf.iter().map(|s| s.norm_sqr()).sum();
        assert!(energy > 0.0);

        sdr.stop_rx().unwrap();
        assert!(!sdr.is_active());
    }

    #[test]
    fn test_pluto_iio_endpoint_url_parsing() {
        let ip_res = parse_iio_uri("ip:192.168.1.10");
        assert!(ip_res.is_ok());
        if let Ok(IioTransport::Network(addr)) = ip_res {
            assert_eq!(addr.ip().to_string(), "192.168.1.10");
            assert_eq!(addr.port(), 50901);
        } else {
            panic!("Expected Network transport");
        }

        let usb_res = parse_iio_uri("usb:1.4");
        assert!(usb_res.is_ok());
        assert_eq!(usb_res.unwrap(), IioTransport::Usb { bus: 1, address: 4 });

        let local_res = parse_iio_uri("local:");
        assert_eq!(local_res.unwrap(), IioTransport::Local);

        let invalid = parse_iio_uri("invalid_scheme://host");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_sigmf_roundtrip_metadata_and_samples() {
        let temp_dir = std::env::temp_dir();
        let base_path = temp_dir.join("test_sdr_dataset");

        let sample_rate = 2400000.0;
        let center_freq = 1090.0e6;
        let mut writer = SigMfWriter::create(&base_path, sample_rate, center_freq).unwrap();

        let samples = vec![
            Complex32::new(0.5, -0.5),
            Complex32::new(-0.25, 0.75),
            Complex32::new(1.0, 0.0),
        ];
        writer.write_samples(&samples).unwrap();
        writer.close().unwrap();

        let mut reader = SigMfReader::open(&base_path).unwrap();
        assert_eq!(reader.metadata.global.sample_rate, 2400000.0);
        assert_eq!(reader.metadata.captures[0].frequency, 1090.0e6);

        let mut read_buf = vec![Complex32::default(); 3];
        let n = reader.read_samples(&mut read_buf).unwrap();
        assert_eq!(n, 3);
        for (orig, read) in samples.iter().zip(read_buf.iter()) {
            assert!((orig.re - read.re).abs() < 1e-5);
            assert!((orig.im - read.im).abs() < 1e-5);
        }

        // Clean up temporary files
        let _ = std::fs::remove_file(base_path.with_extension("sigmf-meta"));
        let _ = std::fs::remove_file(base_path.with_extension("sigmf-data"));
    }

    #[test]
    fn test_wav_reader_writer() {
        let temp_dir = std::env::temp_dir();
        let wav_path = temp_dir.join("test_sdr_iq.wav");

        let samples = vec![
            Complex32::new(0.5, -0.5),
            Complex32::new(-0.25, 0.75),
            Complex32::new(0.9, -0.1),
        ];

        write_iq_wav(&wav_path, 48000, &samples).unwrap();
        let (rate, loaded) = read_iq_wav(&wav_path).unwrap();
        assert_eq!(rate, 48000);
        assert_eq!(loaded.len(), samples.len());

        for (orig, read) in samples.iter().zip(loaded.iter()) {
            assert!((orig.re - read.re).abs() < 1e-3);
            assert!((orig.im - read.im).abs() < 1e-3);
        }

        let _ = std::fs::remove_file(wav_path);
    }
}
