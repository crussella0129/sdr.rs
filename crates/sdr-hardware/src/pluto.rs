//! PlutoSDR / Pluto+ driver over the libiio network daemon (iiod).
//!
//! Real RX I/O is performed by [`crate::iiod::IiodClient`], a pure-Rust iiod
//! network-protocol client (no `libiio`/SoapySDR C dependency). The AD9361/AD9363
//! transceiver is controlled through `ad9361-phy` attributes and RX IQ is streamed
//! from `cf-ad9361-lpc`. TX is not yet implemented.

use crate::driver::{DeviceInfo, GainMode, SdrDriver};
use crate::iiod::{iq_bytes_to_complex32, parse_context_devices, Direction, IiodClient, IIOD_PORT};
use sdr_core::sample::Complex32;
use sdr_core::traits::{Result, SdrError};
use std::net::SocketAddr;
use std::time::Duration;

/// IIO transport types for PlutoSDR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IioTransport {
    Network(SocketAddr),
    Usb { bus: u8, address: u8 },
    Local,
}

// RX signal-path locations on the AD9361 (`ad9361-phy`) control device.
const RX_LO_CHANNEL: &str = "altvoltage0"; // OUTPUT, attr `frequency`
const RX_CHANNEL: &str = "voltage0"; // INPUT: sampling_frequency / rf_bandwidth / hardwaregain / gain_control_mode
                                     // RX ADC scan channels I(0) + Q(1) enabled.
const RX_CHANNEL_MASK: u32 = 0b11;
// Fallback device ids if the context XML cannot be parsed.
const DEFAULT_PHY_DEV: &str = "iio:device0"; // ad9361-phy
const DEFAULT_RX_DEV: &str = "iio:device3"; // cf-ad9361-lpc

/// PlutoSDR / Pluto+ driver for AD9361/AD9363 RF transceivers.
pub struct PlutoSdr {
    uri: String,
    transport: IioTransport,
    addr: Option<SocketAddr>,
    frequency: f64,
    sample_rate: f64,
    bandwidth: f64,
    gain_db: f64,
    gain_mode: GainMode,
    active: bool,
    client: Option<IiodClient>,
    phy_dev: String,
    rx_dev: String,
    rx_open_samples: Option<usize>,
}

impl PlutoSdr {
    /// Parse an IIO URI (e.g. `ip:192.168.2.1`, `ip:192.168.2.1:30431`,
    /// `usb:1.2`, `local:`) and create the driver.
    pub fn new(uri: &str) -> Result<Self> {
        let transport = parse_iio_uri(uri)?;
        let addr = match transport {
            IioTransport::Network(a) => Some(a),
            _ => None,
        };
        Ok(Self {
            uri: uri.to_string(),
            transport,
            addr,
            frequency: 915.0e6,
            sample_rate: 2.0e6,
            bandwidth: 2.0e6,
            gain_db: 40.0,
            gain_mode: GainMode::Manual,
            active: false,
            client: None,
            phy_dev: DEFAULT_PHY_DEV.to_string(),
            rx_dev: DEFAULT_RX_DEV.to_string(),
            rx_open_samples: None,
        })
    }

    /// Default network Pluto+ configuration (USB-Ethernet gadget `192.168.2.1`).
    pub fn default_network() -> Result<Self> {
        Self::new("ip:192.168.2.1")
    }

    /// IIO endpoint URI.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Transport interface for this driver instance.
    pub fn transport(&self) -> &IioTransport {
        &self.transport
    }

    /// Connect to the iiod endpoint, negotiate the version, and resolve the
    /// AD9361 control and RX streaming device ids from the context.
    pub fn connect(&mut self) -> Result<()> {
        let addr = self.addr.ok_or_else(|| {
            SdrError::Config(format!(
                "PlutoSDR '{}' requires a network endpoint (ip:<host>); \
                 USB/local iiod transports are not supported by the pure-Rust client",
                self.uri
            ))
        })?;
        let mut client = IiodClient::connect(addr)?;
        let version = client.version()?;
        log::info!("Connected to iiod {version} at {addr}");

        match client.print_context() {
            Ok(xml) => {
                let devices = parse_context_devices(&xml);
                if let Some(phy) = devices.iter().find(|d| d.name == "ad9361-phy") {
                    self.phy_dev = phy.id.clone();
                }
                if let Some(rx) = devices.iter().find(|d| d.name == "cf-ad9361-lpc") {
                    self.rx_dev = rx.id.clone();
                }
            }
            Err(e) => log::warn!("iiod PRINT failed ({e}); using default device ids"),
        }

        self.client = Some(client);
        Ok(())
    }

    /// Write the currently-configured tuning to the connected device.
    fn apply_settings(&mut self) -> Result<()> {
        let (rate, bw, gain, mode, freq) = (
            self.sample_rate,
            self.bandwidth,
            self.gain_db,
            self.gain_mode,
            self.frequency,
        );
        let phy = self.phy_dev.clone();
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| SdrError::Hardware("PlutoSDR not connected".to_string()))?;
        client.write_channel_attr(
            &phy,
            Direction::Input,
            RX_CHANNEL,
            "sampling_frequency",
            &format!("{}", rate as u64),
        )?;
        client.write_channel_attr(
            &phy,
            Direction::Input,
            RX_CHANNEL,
            "rf_bandwidth",
            &format!("{}", bw as u64),
        )?;
        client.write_channel_attr(
            &phy,
            Direction::Input,
            RX_CHANNEL,
            "gain_control_mode",
            gain_mode_str(mode),
        )?;
        if mode == GainMode::Manual {
            client.write_channel_attr(
                &phy,
                Direction::Input,
                RX_CHANNEL,
                "hardwaregain",
                &format!("{:.6}", gain),
            )?;
        }
        client.write_channel_attr(
            &phy,
            Direction::Output,
            RX_LO_CHANNEL,
            "frequency",
            &format!("{}", freq as u64),
        )?;
        Ok(())
    }
}

/// Map a [`GainMode`] to the AD9361 `gain_control_mode` attribute value.
fn gain_mode_str(mode: GainMode) -> &'static str {
    match mode {
        GainMode::Manual => "manual",
        GainMode::FastAttack => "fast_attack",
        GainMode::SlowAttack => "slow_attack",
        GainMode::Hybrid => "hybrid",
    }
}

impl SdrDriver for PlutoSdr {
    fn name(&self) -> &str {
        "PlutoSDR / AD9363 iiod Driver"
    }

    fn set_frequency(&mut self, _channel: usize, freq_hz: f64) -> Result<()> {
        if !(70.0e6..=6.0e9).contains(&freq_hz) {
            return Err(SdrError::Config(format!(
                "Frequency {:.2} MHz out of PlutoSDR range (70 MHz - 6 GHz)",
                freq_hz / 1e6
            )));
        }
        self.frequency = freq_hz;
        let phy = self.phy_dev.clone();
        if let Some(client) = self.client.as_mut() {
            client.write_channel_attr(
                &phy,
                Direction::Output,
                RX_LO_CHANNEL,
                "frequency",
                &format!("{}", freq_hz as u64),
            )?;
        }
        Ok(())
    }

    fn set_sample_rate(&mut self, _channel: usize, rate_hz: f64) -> Result<()> {
        if !(65105.0..=61.44e6).contains(&rate_hz) {
            return Err(SdrError::Config(format!(
                "Sample rate {:.2} kSPS out of PlutoSDR range (65.1 kSPS - 61.44 MSPS)",
                rate_hz / 1e3
            )));
        }
        self.sample_rate = rate_hz;
        let phy = self.phy_dev.clone();
        if let Some(client) = self.client.as_mut() {
            client.write_channel_attr(
                &phy,
                Direction::Input,
                RX_CHANNEL,
                "sampling_frequency",
                &format!("{}", rate_hz as u64),
            )?;
        }
        Ok(())
    }

    fn set_bandwidth(&mut self, _channel: usize, bw_hz: f64) -> Result<()> {
        if !(200.0e3..=56.0e6).contains(&bw_hz) {
            return Err(SdrError::Config(format!(
                "Bandwidth {:.2} MHz out of range (200 kHz - 56 MHz)",
                bw_hz / 1e6
            )));
        }
        self.bandwidth = bw_hz;
        let phy = self.phy_dev.clone();
        if let Some(client) = self.client.as_mut() {
            client.write_channel_attr(
                &phy,
                Direction::Input,
                RX_CHANNEL,
                "rf_bandwidth",
                &format!("{}", bw_hz as u64),
            )?;
        }
        Ok(())
    }

    fn set_gain(&mut self, _channel: usize, gain_db: f64) -> Result<()> {
        if !(0.0..=73.0).contains(&gain_db) {
            return Err(SdrError::Config(format!(
                "Gain {gain_db:.1} dB out of PlutoSDR range (0 to 73 dB)"
            )));
        }
        self.gain_db = gain_db;
        let phy = self.phy_dev.clone();
        if let Some(client) = self.client.as_mut() {
            client.write_channel_attr(
                &phy,
                Direction::Input,
                RX_CHANNEL,
                "hardwaregain",
                &format!("{gain_db:.6}"),
            )?;
        }
        Ok(())
    }

    fn set_gain_mode(&mut self, _channel: usize, mode: GainMode) -> Result<()> {
        self.gain_mode = mode;
        let phy = self.phy_dev.clone();
        if let Some(client) = self.client.as_mut() {
            client.write_channel_attr(
                &phy,
                Direction::Input,
                RX_CHANNEL,
                "gain_control_mode",
                gain_mode_str(mode),
            )?;
        }
        Ok(())
    }

    fn start_rx(&mut self) -> Result<()> {
        if self.client.is_none() {
            self.connect()?;
        }
        self.apply_settings()?;
        self.active = true;
        log::info!("PlutoSDR RX streaming started");
        Ok(())
    }

    fn stop_rx(&mut self) -> Result<()> {
        if self.rx_open_samples.is_some() {
            let rx_dev = self.rx_dev.clone();
            if let Some(client) = self.client.as_mut() {
                let _ = client.close(&rx_dev);
            }
            self.rx_open_samples = None;
        }
        self.active = false;
        log::info!("PlutoSDR RX streaming stopped");
        Ok(())
    }

    fn read_samples(&mut self, buffer: &mut [Complex32]) -> Result<usize> {
        if !self.active {
            return Ok(0);
        }
        let want = buffer.len();
        if want == 0 {
            return Ok(0);
        }
        let rx_dev = self.rx_dev.clone();
        let need_open = self.rx_open_samples != Some(want);
        let had_open = self.rx_open_samples.is_some();

        let client = self
            .client
            .as_mut()
            .ok_or_else(|| SdrError::Hardware("PlutoSDR not connected".to_string()))?;
        if need_open {
            if had_open {
                let _ = client.close(&rx_dev);
            }
            client.open(&rx_dev, want, RX_CHANNEL_MASK)?;
        }
        let bytes = client.read_buf(&rx_dev, want * 4)?;

        self.rx_open_samples = Some(want);
        let iq = iq_bytes_to_complex32(&bytes);
        let n = iq.len().min(buffer.len());
        buffer[..n].copy_from_slice(&iq[..n]);
        Ok(n)
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn teardown(&mut self) -> Result<()> {
        if self.rx_open_samples.is_some() {
            let rx_dev = self.rx_dev.clone();
            if let Some(client) = self.client.as_mut() {
                let _ = client.close(&rx_dev);
            }
            self.rx_open_samples = None;
        }
        self.client = None;
        self.active = false;
        Ok(())
    }
}

/// Best-effort probe of an iiod endpoint, returning a [`DeviceInfo`] if a
/// PlutoSDR (`ad9361-phy`) is present. Used by device enumeration; `timeout`
/// keeps the probe from blocking when nothing is connected.
pub fn probe_devices(uri: &str, timeout: Duration) -> Result<Vec<DeviceInfo>> {
    let addr = match parse_iio_uri(uri)? {
        IioTransport::Network(a) => a,
        _ => {
            return Err(SdrError::Config(
                "device probe requires a network (ip:) endpoint".to_string(),
            ))
        }
    };
    let mut client = IiodClient::connect_with_timeout(addr, timeout)?;
    client.version()?;
    let xml = client.print_context()?;
    let devices = parse_context_devices(&xml);
    if devices.iter().any(|d| d.name == "ad9361-phy") {
        // The RX capture device (`cf-ad9361-lpc`) exposes I/Q as paired scan
        // channels, so channel-pairs map to one RX/TX stream.
        let rx_pairs = devices
            .iter()
            .find(|d| d.name == "cf-ad9361-lpc")
            .map(|d| (d.input_channels / 2).max(1))
            .unwrap_or(1);
        Ok(vec![DeviceInfo {
            name: "PlutoSDR / AD936x".to_string(),
            serial: None,
            uri: uri.to_string(),
            rx_channels: rx_pairs,
            tx_channels: 1,
        }])
    } else {
        Ok(Vec::new())
    }
}

/// Parse standard IIO URI strings into an [`IioTransport`].
///
/// Network URIs without an explicit port default to the iiod port
/// ([`IIOD_PORT`]).
pub fn parse_iio_uri(uri: &str) -> Result<IioTransport> {
    if uri == "local:" || uri == "local" {
        return Ok(IioTransport::Local);
    }

    if let Some(rest) = uri.strip_prefix("ip:") {
        let addr_str = if rest.contains(':') {
            rest.to_string()
        } else {
            format!("{rest}:{IIOD_PORT}")
        };
        let socket_addr = addr_str
            .parse::<SocketAddr>()
            .map_err(|e| SdrError::Config(format!("Invalid IP endpoint '{uri}': {e}")))?;
        return Ok(IioTransport::Network(socket_addr));
    }

    if let Some(rest) = uri.strip_prefix("usb:") {
        let parts: Vec<&str> = rest.split('.').collect();
        if parts.len() >= 2 {
            let bus = parts[0]
                .parse::<u8>()
                .map_err(|_| SdrError::Config(format!("Invalid USB bus in '{uri}'")))?;
            let address = parts[1]
                .parse::<u8>()
                .map_err(|_| SdrError::Config(format!("Invalid USB address in '{uri}'")))?;
            return Ok(IioTransport::Usb { bus, address });
        }
    }

    Err(SdrError::Config(format!(
        "Unrecognized IIO URI format: '{uri}'. Expected 'ip:<host>', 'usb:<bus>.<addr>', or 'local:'"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pluto_defaults_addr_port() {
        let pluto = PlutoSdr::default_network().unwrap();
        assert_eq!(pluto.uri(), "ip:192.168.2.1");
        match pluto.transport() {
            IioTransport::Network(addr) => {
                assert_eq!(addr.ip().to_string(), "192.168.2.1");
                assert_eq!(addr.port(), IIOD_PORT);
            }
            other => panic!("expected network transport, got {other:?}"),
        }
    }

    #[test]
    fn test_pluto_uri_explicit_port_preserved() {
        let pluto = PlutoSdr::new("ip:192.168.2.1:1234").unwrap();
        match pluto.transport() {
            IioTransport::Network(addr) => assert_eq!(addr.port(), 1234),
            other => panic!("expected network transport, got {other:?}"),
        }
    }

    #[test]
    fn test_pluto_rejects_out_of_range_frequency() {
        let mut pluto = PlutoSdr::default_network().unwrap();
        assert!(pluto.set_frequency(0, 10.0e6).is_err());
        assert!(pluto.set_frequency(0, 100.0e6).is_ok());
    }
}
