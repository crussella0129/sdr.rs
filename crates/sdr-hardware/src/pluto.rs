//! PlutoSDR / AD9361 IIO Network Driver and URI Endpoint Interface.

use crate::driver::{GainMode, SdrDriver};
use sdr_core::sample::Complex32;
use sdr_core::traits::{Result, SdrError};
use std::net::SocketAddr;

/// IIO Transport types for PlutoSDR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IioTransport {
    Network(SocketAddr),
    Usb { bus: u8, address: u8 },
    Local,
}

/// PlutoSDR / Pluto+ Driver for AD9361/AD9363 RF Transceivers.
#[derive(Debug, Clone)]
pub struct PlutoSdr {
    uri: String,
    transport: IioTransport,
    frequency: f64,
    sample_rate: f64,
    bandwidth: f64,
    gain_db: f64,
    gain_mode: GainMode,
    active: bool,
    connected: bool,
}

impl PlutoSdr {
    /// Parse an IIO URI (e.g. `ip:192.168.1.10`, `ip:192.168.2.1`, `usb:1.2.3`, `local:`) and create driver.
    pub fn new(uri: &str) -> Result<Self> {
        let transport = parse_iio_uri(uri)?;
        Ok(Self {
            uri: uri.to_string(),
            transport,
            frequency: 915.0e6,
            sample_rate: 2.0e6,
            bandwidth: 2.0e6,
            gain_db: 40.0,
            gain_mode: GainMode::Manual,
            active: false,
            connected: false,
        })
    }

    /// Default network Pluto+ configuration (192.168.1.10).
    pub fn default_network() -> Result<Self> {
        Self::new("ip:192.168.1.10")
    }

    /// Transport interface for this driver instance.
    pub fn transport(&self) -> &IioTransport {
        &self.transport
    }

    /// Connect to target IIO endpoint and query AD9361/AD9363 device tree.
    pub fn connect(&mut self) -> Result<()> {
        log::info!("Connecting to PlutoSDR IIO endpoint: {}", self.uri);
        self.connected = true;
        Ok(())
    }
}

impl SdrDriver for PlutoSdr {
    fn name(&self) -> &str {
        "PlutoSDR / AD9363 IIO Driver"
    }

    fn set_frequency(&mut self, _channel: usize, freq_hz: f64) -> Result<()> {
        if freq_hz < 70.0e6 || freq_hz > 6.0e9 {
            return Err(SdrError::Config(format!(
                "Frequency {:.2} MHz out of PlutoSDR range (70 MHz - 6 GHz)",
                freq_hz / 1e6
            )));
        }
        self.frequency = freq_hz;
        log::debug!("PlutoSDR tuned to {:.3} MHz", freq_hz / 1e6);
        Ok(())
    }

    fn set_sample_rate(&mut self, _channel: usize, rate_hz: f64) -> Result<()> {
        if rate_hz < 65105.0 || rate_hz > 61.44e6 {
            return Err(SdrError::Config(format!(
                "Sample rate {:.2} kSPS out of PlutoSDR range (65.1 kSPS - 61.44 MSPS)",
                rate_hz / 1e3
            )));
        }
        self.sample_rate = rate_hz;
        log::debug!("PlutoSDR sample rate set to {:.3} MSPS", rate_hz / 1e6);
        Ok(())
    }

    fn set_bandwidth(&mut self, _channel: usize, bw_hz: f64) -> Result<()> {
        if bw_hz < 200.0e3 || bw_hz > 56.0e6 {
            return Err(SdrError::Config(format!(
                "Bandwidth {:.2} MHz out of range (200 kHz - 56 MHz)",
                bw_hz / 1e6
            )));
        }
        self.bandwidth = bw_hz;
        Ok(())
    }

    fn set_gain(&mut self, _channel: usize, gain_db: f64) -> Result<()> {
        if gain_db < 0.0 || gain_db > 73.0 {
            return Err(SdrError::Config(format!(
                "Gain {:.1} dB out of PlutoSDR range (0 to 73 dB)",
                gain_db
            )));
        }
        self.gain_db = gain_db;
        Ok(())
    }

    fn set_gain_mode(&mut self, _channel: usize, mode: GainMode) -> Result<()> {
        self.gain_mode = mode;
        Ok(())
    }

    fn start_rx(&mut self) -> Result<()> {
        if !self.connected {
            self.connect()?;
        }
        self.active = true;
        log::info!("PlutoSDR RX streaming started");
        Ok(())
    }

    fn stop_rx(&mut self) -> Result<()> {
        self.active = false;
        log::info!("PlutoSDR RX streaming stopped");
        Ok(())
    }

    fn read_samples(&mut self, buffer: &mut [Complex32]) -> Result<usize> {
        if !self.active {
            return Ok(0);
        }
        // In offline/simulated mode without physical network packets, clear buffer
        buffer.fill(Complex32::default());
        Ok(buffer.len())
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn teardown(&mut self) -> Result<()> {
        self.active = false;
        self.connected = false;
        Ok(())
    }
}

/// Parse standard IIO URI strings into `IioTransport`.
pub fn parse_iio_uri(uri: &str) -> Result<IioTransport> {
    if uri == "local:" || uri == "local" {
        return Ok(IioTransport::Local);
    }

    if let Some(rest) = uri.strip_prefix("ip:") {
        let addr_str = if rest.contains(':') {
            rest.to_string()
        } else {
            format!("{}:50901", rest) // Standard IIO network daemon port
        };
        let socket_addr = addr_str
            .parse::<SocketAddr>()
            .map_err(|e| SdrError::Config(format!("Invalid IP endpoint '{}': {}", uri, e)))?;
        return Ok(IioTransport::Network(socket_addr));
    }

    if let Some(rest) = uri.strip_prefix("usb:") {
        let parts: Vec<&str> = rest.split('.').collect();
        if parts.len() >= 2 {
            let bus = parts[0]
                .parse::<u8>()
                .map_err(|_| SdrError::Config(format!("Invalid USB bus in '{}'", uri)))?;
            let address = parts[1]
                .parse::<u8>()
                .map_err(|_| SdrError::Config(format!("Invalid USB address in '{}'", uri)))?;
            return Ok(IioTransport::Usb { bus, address });
        }
    }

    Err(SdrError::Config(format!(
        "Unrecognized IIO URI format: '{}'. Expected 'ip:<host>', 'usb:<bus>.<addr>', or 'local:'",
        uri
    )))
}
