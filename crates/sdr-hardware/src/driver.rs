//! Common Hardware Abstraction Layer (HAL) for SDR devices.

use sdr_core::sample::Complex32;
use sdr_core::traits::Result;

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

    /// Check if receiver streaming is actively running.
    fn is_active(&self) -> bool;

    /// Cleanly close device connections and release resources.
    fn teardown(&mut self) -> Result<()>;
}
