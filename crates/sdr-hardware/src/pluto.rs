//! PlutoSDR / Pluto+ driver over the libiio network daemon (iiod).
//!
//! Real I/O is performed by [`crate::iiod::IiodClient`], a pure-Rust iiod
//! network-protocol client (no `libiio`/SoapySDR C dependency). The AD9361/AD9363
//! transceiver is controlled through `ad9361-phy` attributes; RX IQ is streamed
//! from `cf-ad9361-lpc` and TX IQ is streamed to `cf-ad9361-dds-core-lpc`.
//!
//! # Transmitting responsibly
//!
//! [`SdrDriver::write_samples`] drives a real transmitter. It is inert until
//! [`SdrDriver::start_tx`] is called, and the driver starts at maximum
//! attenuation. For live testing, use
//! [`PlutoSdr::enter_loopback_test_mode`], which bypasses the RF section
//! entirely; only [`LoopbackMode`] variants that keep the signal internal are
//! representable.

use crate::driver::{DeviceInfo, GainMode, SdrDriver};
use crate::iiod::{
    complex32_to_iq_bytes, iq_bytes_to_complex32, parse_context_devices, Direction, IiodClient,
    IIOD_PORT,
};
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

// TX signal-path locations on the same control device.
const TX_LO_CHANNEL: &str = "altvoltage1"; // OUTPUT, attr `frequency`
const TX_CHANNEL: &str = "voltage0"; // OUTPUT: sampling_frequency / rf_bandwidth / hardwaregain
                                     // TX DAC scan channels I(0) + Q(1) enabled.
const TX_CHANNEL_MASK: u32 = 0b11;

/// AD9361 debug attribute controlling the internal loopback path.
const LOOPBACK_ATTR: &str = "loopback";

/// DDS tone-generator channels on `cf-ad9361-dds-core-lpc` (`altvoltage0..3`).
const DDS_TONE_CHANNELS: usize = 4;

/// Minimum TX `hardwaregain` in dB — i.e. **maximum attenuation**, the quietest
/// the transmitter can be driven. Probed from the live device
/// (`hardwaregain_available` = `[-89.750000 0.250000 0.000000]`).
pub const TX_GAIN_MIN_DB: f64 = -89.75;
/// Maximum TX `hardwaregain` in dB (0 dB attenuation = full output).
pub const TX_GAIN_MAX_DB: f64 = 0.0;

// Fallback device ids if the context XML cannot be parsed.
const DEFAULT_PHY_DEV: &str = "iio:device0"; // ad9361-phy
const DEFAULT_RX_DEV: &str = "iio:device3"; // cf-ad9361-lpc
const DEFAULT_TX_DEV: &str = "iio:device2"; // cf-ad9361-dds-core-lpc

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
    tx_frequency: f64,
    tx_gain_db: f64,
    tx_active: bool,
    tx_dev: String,
    tx_open_samples: Option<usize>,
    tx_cyclic: bool,
    /// Device state saved by [`PlutoSdr::enter_loopback_test_mode`] so it can be
    /// restored on exit.
    saved_state: Option<SavedTxState>,
}

/// TX-related device state captured before entering loopback test mode.
#[derive(Debug, Clone)]
struct SavedTxState {
    loopback: String,
    tx_gain: String,
    dds_enabled: bool,
}

/// AD9361 internal loopback selection.
///
/// The transceiver also supports an FPGA-internal RX→TX mode (`loopback=2`) in
/// which **the RF chain is active and the device transmits**. That mode is
/// deliberately not representable here: this API can only select paths that do
/// keep the signal internal, so a caller cannot accidentally key the transmitter through it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopbackMode {
    /// Normal operation — no loopback.
    Disabled,
    /// AD9361-internal digital TX→RX loopback. The entire RF section is
    /// bypassed, so nothing is transmitted over the air.
    InternalDigital,
}

impl LoopbackMode {
    /// The `loopback` debug-attribute value for this mode.
    pub fn as_str(self) -> &'static str {
        match self {
            LoopbackMode::Disabled => "0",
            LoopbackMode::InternalDigital => "1",
        }
    }
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
            tx_frequency: 915.0e6,
            tx_gain_db: TX_GAIN_MIN_DB,
            tx_active: false,
            tx_dev: DEFAULT_TX_DEV.to_string(),
            tx_open_samples: None,
            tx_cyclic: false,
            saved_state: None,
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
                if let Some(tx) = devices.iter().find(|d| d.name == "cf-ad9361-dds-core-lpc") {
                    self.tx_dev = tx.id.clone();
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

    /// Set the transmit LO frequency in Hz.
    pub fn set_tx_frequency(&mut self, freq_hz: f64) -> Result<()> {
        if !(70.0e6..=6.0e9).contains(&freq_hz) {
            return Err(SdrError::Config(format!(
                "TX frequency {:.2} MHz out of PlutoSDR range (70 MHz - 6 GHz)",
                freq_hz / 1e6
            )));
        }
        self.tx_frequency = freq_hz;
        let phy = self.phy_dev.clone();
        if let Some(client) = self.client.as_mut() {
            client.write_channel_attr(
                &phy,
                Direction::Output,
                TX_LO_CHANNEL,
                "frequency",
                &format!("{}", freq_hz as u64),
            )?;
        }
        Ok(())
    }

    /// Set the transmit gain in dB.
    ///
    /// On the AD936x this attribute is **attenuation**: `0.0` is full output and
    /// [`TX_GAIN_MIN_DB`] (−89.75 dB) is the quietest setting. Values outside
    /// that range are rejected without touching the device.
    pub fn set_tx_gain(&mut self, gain_db: f64) -> Result<()> {
        if !(TX_GAIN_MIN_DB..=TX_GAIN_MAX_DB).contains(&gain_db) {
            return Err(SdrError::Config(format!(
                "TX gain {gain_db:.2} dB out of PlutoSDR range ({TX_GAIN_MIN_DB} to {TX_GAIN_MAX_DB} dB attenuation)"
            )));
        }
        self.tx_gain_db = gain_db;
        let phy = self.phy_dev.clone();
        if let Some(client) = self.client.as_mut() {
            client.write_channel_attr(
                &phy,
                Direction::Output,
                TX_CHANNEL,
                "hardwaregain",
                &format!("{gain_db:.6}"),
            )?;
        }
        Ok(())
    }

    /// Write the currently-configured transmit tuning to the connected device.
    fn apply_tx_settings(&mut self) -> Result<()> {
        let (rate, bw, gain, freq) = (
            self.sample_rate,
            self.bandwidth,
            self.tx_gain_db,
            self.tx_frequency,
        );
        let phy = self.phy_dev.clone();
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| SdrError::Hardware("PlutoSDR not connected".to_string()))?;
        client.write_channel_attr(
            &phy,
            Direction::Output,
            TX_CHANNEL,
            "sampling_frequency",
            &format!("{}", rate as u64),
        )?;
        client.write_channel_attr(
            &phy,
            Direction::Output,
            TX_CHANNEL,
            "rf_bandwidth",
            &format!("{}", bw as u64),
        )?;
        client.write_channel_attr(
            &phy,
            Direction::Output,
            TX_CHANNEL,
            "hardwaregain",
            &format!("{gain:.6}"),
        )?;
        client.write_channel_attr(
            &phy,
            Direction::Output,
            TX_LO_CHANNEL,
            "frequency",
            &format!("{}", freq as u64),
        )?;
        Ok(())
    }

    /// Select whether the TX buffer is cyclic.
    ///
    /// A cyclic buffer repeats its contents continuously until closed, which is
    /// how a transmitter sustains a waveform; a one-shot buffer drains as soon
    /// as it is consumed. Takes effect the next time the TX buffer is opened.
    pub fn set_tx_cyclic(&mut self, cyclic: bool) {
        if self.tx_cyclic != cyclic {
            self.tx_cyclic = cyclic;
            // Force a reopen so the new mode applies.
            self.close_tx_buffer();
        }
    }

    /// Enable or disable the DDS tone generators on the TX device.
    ///
    /// The `cf-ad9361-dds-core-lpc` core ships with its tone generators **on**.
    /// They and buffer-based transmission are mutually exclusive: while the DDS
    /// is enabled the DAC emits its own tones rather than the samples written
    /// through [`SdrDriver::write_samples`], so [`SdrDriver::start_tx`] turns it
    /// off.
    pub fn set_dds_enabled(&mut self, enabled: bool) -> Result<()> {
        let tx_dev = self.tx_dev.clone();
        let value = if enabled { "1" } else { "0" };
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| SdrError::Hardware("PlutoSDR not connected".to_string()))?;
        for ch in 0..DDS_TONE_CHANNELS {
            client.write_channel_attr(
                &tx_dev,
                Direction::Output,
                &format!("altvoltage{ch}"),
                "raw",
                value,
            )?;
        }
        log::debug!(
            "PlutoSDR DDS tone generators {}",
            if enabled { "enabled" } else { "disabled" }
        );
        Ok(())
    }

    /// Select the AD9361 internal loopback path.
    pub fn set_loopback(&mut self, mode: LoopbackMode) -> Result<()> {
        let phy = self.phy_dev.clone();
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| SdrError::Hardware("PlutoSDR not connected".to_string()))?;
        client.write_debug_attr(&phy, LOOPBACK_ATTR, mode.as_str())?;
        log::info!("PlutoSDR loopback set to {mode:?}");
        Ok(())
    }

    /// Put the radio into an **internal-loopback** transmit test configuration.
    ///
    /// Saves the current loopback mode and TX gain, then sets maximum
    /// attenuation ([`TX_GAIN_MIN_DB`]) *before* engaging
    /// [`LoopbackMode::InternalDigital`] — quietest-first, so the transmitter is
    /// already attenuated whatever happens next. With the internal digital
    /// loopback engaged the RF section is bypassed entirely, so transmitted
    /// samples return on the RX path internally.
    ///
    /// Pair with [`PlutoSdr::exit_loopback_test_mode`] to restore the prior state.
    pub fn enter_loopback_test_mode(&mut self) -> Result<()> {
        if self.client.is_none() {
            self.connect()?;
        }
        let phy = self.phy_dev.clone();
        let tx_dev = self.tx_dev.clone();
        let saved = {
            let client = self
                .client
                .as_mut()
                .ok_or_else(|| SdrError::Hardware("PlutoSDR not connected".to_string()))?;
            SavedTxState {
                loopback: client
                    .read_debug_attr(&phy, LOOPBACK_ATTR)
                    .unwrap_or_else(|_| LoopbackMode::Disabled.as_str().to_string()),
                tx_gain: client
                    .read_channel_attr(&phy, Direction::Output, TX_CHANNEL, "hardwaregain")
                    .unwrap_or_else(|_| format!("{TX_GAIN_MIN_DB}")),
                dds_enabled: client
                    .read_channel_attr(&tx_dev, Direction::Output, "altvoltage0", "raw")
                    .map(|v| v.trim() != "0")
                    .unwrap_or(true),
            }
        };
        self.saved_state = Some(saved);

        // Quietest first: attenuate fully, then bypass the RF section.
        self.set_tx_gain(TX_GAIN_MIN_DB)?;
        self.set_loopback(LoopbackMode::InternalDigital)?;
        // Silence the default DDS tones so a loopback readback reflects the
        // samples actually written, not the core's own generators.
        self.set_dds_enabled(false)?;
        log::info!(
            "PlutoSDR in internal-loopback test mode (RF section bypassed, max attenuation)"
        );
        Ok(())
    }

    /// Restore the loopback mode and TX gain saved by
    /// [`PlutoSdr::enter_loopback_test_mode`].
    pub fn exit_loopback_test_mode(&mut self) -> Result<()> {
        let Some(saved) = self.saved_state.take() else {
            return Ok(());
        };
        self.set_dds_enabled(saved.dds_enabled)?;
        let phy = self.phy_dev.clone();
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| SdrError::Hardware("PlutoSDR not connected".to_string()))?;
        client.write_debug_attr(&phy, LOOPBACK_ATTR, &saved.loopback)?;
        // The saved gain reads back as e.g. "-10.000000 dB"; send only the value.
        let gain_value = saved
            .tx_gain
            .split_whitespace()
            .next()
            .unwrap_or(&saved.tx_gain)
            .to_string();
        client.write_channel_attr(
            &phy,
            Direction::Output,
            TX_CHANNEL,
            "hardwaregain",
            &gain_value,
        )?;
        log::info!("PlutoSDR loopback test mode exited; prior TX state restored");
        Ok(())
    }

    /// Close the TX buffer if one is open, ignoring a failure to close.
    fn close_tx_buffer(&mut self) {
        if self.tx_open_samples.is_some() {
            let tx_dev = self.tx_dev.clone();
            if let Some(client) = self.client.as_mut() {
                let _ = client.close(&tx_dev);
            }
            self.tx_open_samples = None;
        }
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

    fn has_tx(&self) -> bool {
        true
    }

    fn start_tx(&mut self) -> Result<()> {
        if self.client.is_none() {
            self.connect()?;
        }
        self.apply_tx_settings()?;
        // The DDS tone generators are enabled by default and would otherwise be
        // transmitted instead of the caller's samples.
        self.set_dds_enabled(false)?;
        self.tx_active = true;
        log::info!(
            "PlutoSDR TX streaming started ({:.3} MHz, gain {:.2} dB)",
            self.tx_frequency / 1e6,
            self.tx_gain_db
        );
        Ok(())
    }

    fn stop_tx(&mut self) -> Result<()> {
        self.close_tx_buffer();
        self.tx_active = false;
        log::info!("PlutoSDR TX streaming stopped");
        Ok(())
    }

    /// Transmit IQ samples.
    ///
    /// Returns `Ok(0)` without touching the radio unless [`SdrDriver::start_tx`]
    /// has been called, so an accidental write cannot key the transmitter.
    fn write_samples(&mut self, buffer: &[Complex32]) -> Result<usize> {
        if !self.tx_active || buffer.is_empty() {
            return Ok(0);
        }
        let want = buffer.len();
        let tx_dev = self.tx_dev.clone();
        let need_open = self.tx_open_samples != Some(want);
        let had_open = self.tx_open_samples.is_some();
        let cyclic = self.tx_cyclic;
        let bytes = complex32_to_iq_bytes(buffer);

        let client = self
            .client
            .as_mut()
            .ok_or_else(|| SdrError::Hardware("PlutoSDR not connected".to_string()))?;
        if need_open {
            if had_open {
                let _ = client.close(&tx_dev);
            }
            client.open_with(&tx_dev, want, TX_CHANNEL_MASK, cyclic)?;
        }
        let written = client.write_buf(&tx_dev, &bytes)?;

        self.tx_open_samples = Some(want);
        // The daemon reports bytes accepted; report samples to the caller.
        Ok(written / 4)
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
        self.close_tx_buffer();
        self.client = None;
        self.active = false;
        self.tx_active = false;
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
    fn test_pluto_has_tx() {
        let pluto = PlutoSdr::default_network().unwrap();
        assert!(pluto.has_tx(), "PlutoSDR is a transceiver");
    }

    #[test]
    fn test_pluto_tx_gain_range() {
        let mut pluto = PlutoSdr::default_network().unwrap();
        // Bounds are inclusive; the attribute is attenuation, so 0 dB is full output.
        assert!(pluto.set_tx_gain(TX_GAIN_MIN_DB).is_ok());
        assert!(pluto.set_tx_gain(TX_GAIN_MAX_DB).is_ok());
        assert!(pluto.set_tx_gain(-20.0).is_ok());
        // Out of range must be refused (disconnected, so no device write occurs).
        assert!(pluto.set_tx_gain(-100.0).is_err());
        assert!(pluto.set_tx_gain(1.0).is_err());
    }

    #[test]
    fn test_pluto_write_samples_inactive_is_noop() {
        let mut pluto = PlutoSdr::default_network().unwrap();
        // Without start_tx the driver must not transmit, even when disconnected.
        let samples = [Complex32::new(0.5, 0.5); 8];
        assert_eq!(pluto.write_samples(&samples).unwrap(), 0);
    }

    #[test]
    fn test_loopback_mode_values() {
        assert_eq!(LoopbackMode::Disabled.as_str(), "0");
        assert_eq!(LoopbackMode::InternalDigital.as_str(), "1");
    }

    #[test]
    fn test_loopback_mode_excludes_rf() {
        // The FPGA RX->TX mode ("2") actively transmits and must not be
        // reachable through this API: no variant may map to it.
        for mode in [LoopbackMode::Disabled, LoopbackMode::InternalDigital] {
            assert_ne!(
                mode.as_str(),
                "2",
                "the FPGA RX->TX loopback mode must not be constructible"
            );
        }
    }

    #[test]
    fn test_pluto_tx_defaults_to_max_attenuation() {
        // A freshly constructed driver is at the quietest TX setting.
        let pluto = PlutoSdr::default_network().unwrap();
        assert_eq!(pluto.tx_gain_db, TX_GAIN_MIN_DB);
        assert!(!pluto.tx_active);
    }

    #[test]
    fn test_pluto_rejects_out_of_range_frequency() {
        let mut pluto = PlutoSdr::default_network().unwrap();
        assert!(pluto.set_frequency(0, 10.0e6).is_err());
        assert!(pluto.set_frequency(0, 100.0e6).is_ok());
    }
}
