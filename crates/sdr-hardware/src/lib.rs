//! # sdr-hardware
//!
//! Hardware abstraction layer, device drivers (PlutoSDR IIO, Mock), and dataset storage (SigMF, WAV) for `sdr.rs`.

pub mod driver;
pub mod iiod;
pub mod mock;
pub mod pluto;
pub mod sigmf;
pub mod wav;

pub use driver::{DeviceInfo, GainMode, SdrDriver};
pub use iiod::{IiodClient, IiodDeviceInfo};
pub use mock::{MockSdr, MockSignal};
pub use pluto::{LoopbackMode, PlutoSdr};

/// Enumerate available SDR devices across supported backends.
///
/// Always includes the built-in mock device; additionally performs a
/// best-effort probe of the default PlutoSDR network endpoint (`ip:192.168.2.1`)
/// with a short timeout so it never blocks when no radio is attached.
///
/// Additional hardware backends (RTL-SDR, HackRF, Airspy — e.g. via SoapySDR or
/// `seify`) implement the same [`SdrDriver`] trait and plug into this
/// enumeration; that broader multi-vendor backend is a follow-on that requires
/// the corresponding host libraries.
pub fn list_devices() -> Vec<DeviceInfo> {
    let mut devices = vec![DeviceInfo {
        name: "Mock SDR".to_string(),
        serial: None,
        uri: "mock:".to_string(),
        rx_channels: 1,
        tx_channels: 1,
    }];
    if let Ok(mut pluto) =
        pluto::probe_devices("ip:192.168.2.1", std::time::Duration::from_millis(500))
    {
        devices.append(&mut pluto);
    }
    devices
}
pub use sigmf::{SigMfCapture, SigMfGlobal, SigMfMetadata, SigMfReader, SigMfWriter};
pub use wav::{read_iq_wav, write_iq_wav};

#[cfg(test)]
mod tests {
    use super::*;
    use sdr_core::sample::Complex32;

    #[test]
    fn test_list_devices_mock() {
        // The mock device is always enumerated, with or without a radio present.
        let devices = super::list_devices();
        assert!(
            devices.iter().any(|d| d.name == "Mock SDR"),
            "expected a Mock SDR device in enumeration, got {devices:?}"
        );
    }

    #[test]
    fn test_sdr_driver_trait_mock_streaming() {
        let mut sdr = MockSdr::new(2.0e6, 915.0e6);
        assert_eq!(sdr.name(), "Mock SDR Driver");
        assert!(!sdr.is_active());

        sdr.start_rx().unwrap();
        assert!(sdr.is_active());

        let mut buffer = vec![Complex32::default(); 1024];
        let read = sdr.read_samples(&mut buffer).unwrap();
        assert_eq!(read, 1024);

        // Verify non-zero tone samples
        let norm_sum: f32 = buffer.iter().map(|s| s.norm()).sum();
        assert!(norm_sum > 500.0);

        sdr.stop_rx().unwrap();
        assert!(!sdr.is_active());
    }

    #[test]
    fn test_mock_sdr_tx_loopback() {
        let mut sdr = MockSdr::new(1.0e6, 915.0e6);
        assert!(sdr.has_tx());

        sdr.enable_loopback();
        sdr.start_tx().unwrap();
        sdr.start_rx().unwrap();

        let tx_data = vec![
            Complex32::new(1.0, 0.5),
            Complex32::new(-0.5, 0.8),
            Complex32::new(0.3, -0.9),
        ];

        let written = sdr.write_samples(&tx_data).unwrap();
        assert_eq!(written, 3);

        let mut rx_buf = vec![Complex32::default(); 3];
        let read = sdr.read_samples(&mut rx_buf).unwrap();
        assert_eq!(read, 3);
        assert_eq!(rx_buf, tx_data);
    }

    #[test]
    fn test_pluto_iio_endpoint_url_parsing() {
        let uri = "ip:192.168.2.1";
        let pluto = PlutoSdr::new(uri).unwrap();
        assert_eq!(pluto.uri(), uri);
    }

    #[test]
    fn test_sigmf_roundtrip_metadata_and_samples() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_sigmf_capture");

        let sample_rate = 2.4e6;
        let frequency = 433.92e6;

        let samples = vec![
            Complex32::new(1.0, -1.0),
            Complex32::new(0.5, 0.5),
            Complex32::new(-0.5, -0.5),
        ];

        // Write
        let mut writer = SigMfWriter::create(&path, sample_rate, frequency).unwrap();
        writer.write_samples(&samples).unwrap();
        writer.close().unwrap();

        // Read
        let mut reader = SigMfReader::open(&path).unwrap();
        assert_eq!(reader.metadata.global.sample_rate, sample_rate);
        assert_eq!(reader.metadata.captures[0].frequency, frequency);

        let mut read_buf = vec![Complex32::default(); 3];
        let n = reader.read_samples(&mut read_buf).unwrap();
        assert_eq!(n, 3);
        assert_eq!(read_buf, samples);

        // Cleanup
        let _ = std::fs::remove_file(path.with_extension("sigmf-meta"));
        let _ = std::fs::remove_file(path.with_extension("sigmf-data"));
    }

    #[test]
    fn test_wav_reader_writer() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_iq.wav");

        let samples = vec![Complex32::new(0.5, -0.5), Complex32::new(-0.25, 0.25)];

        write_iq_wav(&path, 48000, &samples).unwrap();
        let (rate, read_samples) = read_iq_wav(&path).unwrap();
        assert_eq!(rate, 48000);
        assert_eq!(read_samples.len(), 2);
        assert!((read_samples[0].re - 0.5).abs() < 1e-3);

        let _ = std::fs::remove_file(path);
    }
}
