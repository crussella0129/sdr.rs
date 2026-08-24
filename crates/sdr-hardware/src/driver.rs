//! Common Hardware Abstraction Layer (HAL) for SDR devices.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Result, SdrError};

fn unsupported_transmitter(driver_name: &str) -> SdrError {
    SdrError::Hardware(format!("{driver_name} does not support transmission"))
}

/// AGC (Automatic Gain Control) modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GainMode {
    Manual,
    FastAttack,
    SlowAttack,
    Hybrid,
}

/// Information describing an available SDR device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub serial: Option<String>,
    pub uri: String,
    pub rx_channels: usize,
    pub tx_channels: usize,
}

/// Generic SDR Driver Trait for hardware control and streaming.
pub trait SdrDriver: Send + Sync {
    /// Friendly driver name.
    fn name(&self) -> &str;

    /// Set center RF frequency in Hz for given channel (e.g. 915.0e6).
    fn set_frequency(&mut self, channel: usize, freq_hz: f64) -> Result<()>;

    /// Set baseband sample rate in samples/second (e.g. 2.0e6).
    fn set_sample_rate(&mut self, channel: usize, rate_hz: f64) -> Result<()>;

    /// Set analog baseband RF filter bandwidth in Hz (e.g. 1.5e6).
    fn set_bandwidth(&mut self, channel: usize, bw_hz: f64) -> Result<()>;

    /// Set manual receiver gain in dB (e.g. 40.0).
    fn set_gain(&mut self, channel: usize, gain_db: f64) -> Result<()>;

    /// Set automatic gain control mode.
    fn set_gain_mode(&mut self, channel: usize, mode: GainMode) -> Result<()>;

    /// Start continuous RX sample acquisition.
    fn start_rx(&mut self) -> Result<()>;

    /// Stop RX sample acquisition.
    fn stop_rx(&mut self) -> Result<()>;

    /// Read available IQ samples into destination buffer.
    /// Returns the number of samples read (0 if non-blocking and no samples ready).
    fn read_samples(&mut self, buffer: &mut [Complex32]) -> Result<usize>;

    /// Start continuous TX transmission.
    fn start_tx(&mut self) -> Result<()> {
        Err(unsupported_transmitter(self.name()))
    }

    /// Stop TX transmission.
    fn stop_tx(&mut self) -> Result<()> {
        Err(unsupported_transmitter(self.name()))
    }

    /// Write IQ samples to transmit buffer.
    /// Returns the number of samples queued for transmission.
    fn write_samples(&mut self, _buffer: &[Complex32]) -> Result<usize> {
        Err(unsupported_transmitter(self.name()))
    }

    /// Check whether this SDR device has transmit capability.
    fn has_tx(&self) -> bool {
        false
    }

    /// Check if receiver streaming is actively running.
    fn is_active(&self) -> bool;

    /// Cleanly close device connections and release resources.
    fn teardown(&mut self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RxOnlyDriver;

    impl SdrDriver for RxOnlyDriver {
        fn name(&self) -> &str {
            "RX-only test driver"
        }

        fn set_frequency(&mut self, _channel: usize, _freq_hz: f64) -> Result<()> {
            Ok(())
        }

        fn set_sample_rate(&mut self, _channel: usize, _rate_hz: f64) -> Result<()> {
            Ok(())
        }

        fn set_bandwidth(&mut self, _channel: usize, _bw_hz: f64) -> Result<()> {
            Ok(())
        }

        fn set_gain(&mut self, _channel: usize, _gain_db: f64) -> Result<()> {
            Ok(())
        }

        fn set_gain_mode(&mut self, _channel: usize, _mode: GainMode) -> Result<()> {
            Ok(())
        }

        fn start_rx(&mut self) -> Result<()> {
            Ok(())
        }

        fn stop_rx(&mut self) -> Result<()> {
            Ok(())
        }

        fn read_samples(&mut self, _buffer: &mut [Complex32]) -> Result<usize> {
            Ok(0)
        }

        fn is_active(&self) -> bool {
            false
        }

        fn teardown(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_sdr_driver_default_tx_methods_fail_explicitly() {
        fn assert_unsupported<T: std::fmt::Debug>(result: Result<T>) {
            match result {
                Err(SdrError::Hardware(message)) => {
                    assert!(message.contains("does not support transmission"));
                    assert!(message.contains("RX-only test driver"));
                }
                other => {
                    panic!("expected an unsupported-transmitter hardware error, got {other:?}")
                }
            }
        }

        let mut driver = RxOnlyDriver;
        assert!(!driver.has_tx());
        assert_unsupported(driver.start_tx());
        assert_unsupported(driver.stop_tx());
        assert_unsupported(driver.write_samples(&[Complex32::default()]));
    }
}
