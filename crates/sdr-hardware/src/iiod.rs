//! Pure-Rust client for the libiio network daemon (`iiod`) text protocol.
//!
//! This talks the `iiod` network backend directly over TCP (default port
//! [`IIOD_PORT`]) using only `std::net`, so it needs no `libiio`/SoapySDR C
//! libraries — matching the project's goal of no external C shared-library
//! dependencies on Windows and embedded targets.
//!
//! The wire protocol (verified live against a Pluto+ running iiod 0.21) is a
//! line-oriented command/response protocol:
//!
//! - `VERSION\r\n` → one version line (e.g. `0.21.v0.21`).
//! - `PRINT\r\n` → `<len>\n<len bytes of context XML>`.
//! - `READ <dev> <INPUT|OUTPUT> <chan> <attr>\r\n` → `<len>\n<value+NUL>`;
//!   a negative `<len>` is `-errno`.
//! - `WRITE <dev> <INPUT|OUTPUT> <chan> <attr> <bytelen>\r\n<value+NUL>` →
//!   `<written>\n` (or `-errno`).
//! - `OPEN <dev> <samples> <mask>\r\n` → `<0>\n` (or `-errno`). `<mask>` is a
//!   zero-padded hex bitmask of enabled scan channels, one 32-bit word wide.
//! - `READBUF <dev> <bytes>\r\n` → `<nbytes>\n<mask>\n<nbytes of sample data>`.
//! - `WRITEBUF <dev> <bytes>\r\n` → `<0>\n` (ready ack), then the client sends
//!   the payload and the daemon replies `<written>\n`. Two responses, not one.
//! - `CLOSE <dev>\r\n` → `<0>\n`.
//!
//! Device **debug** attributes use `DEBUG` in the direction slot with no channel
//! name (`READ <dev> DEBUG <attr>`); the no-direction form returns `-2`.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Result, SdrError};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// Standard libiio network daemon (`iiod`) TCP port.
pub const IIOD_PORT: u16 = 30431;

/// Full-scale divisor for AD9361 12-bit signed IQ samples (`S12/16` format).
///
/// The RX capture device (`cf-ad9361-lpc`) reports `le:S12/16>>0`: 12 significant
/// bits carried in a 16-bit word.
const AD9361_RX_FULL_SCALE: f32 = 2048.0;

/// Full-scale multiplier for AD9361 transmit samples (`S16/16` format).
///
/// The TX device (`cf-ad9361-dds-core-lpc`) reports `le:S16/16>>0` — the DAC
/// consumes the **full** 16-bit range, unlike the 12-bit RX path above. Using
/// the RX scale here would transmit at 1/16th amplitude.
const AD9361_TX_FULL_SCALE: f32 = 32768.0;

/// IIO channel direction token used in `READ`/`WRITE` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
    /// Device debug attributes (e.g. `ad9361-phy`'s `loopback`). These are not
    /// channel attributes: the token replaces the direction and no channel name
    /// is supplied.
    Debug,
}

impl Direction {
    fn as_str(self) -> &'static str {
        match self {
            Direction::Input => "INPUT",
            Direction::Output => "OUTPUT",
            Direction::Debug => "DEBUG",
        }
    }
}

// --- Pure command builders (unit-tested without a socket) ---------------------

pub(crate) fn cmd_read_channel(dev: &str, dir: Direction, ch: &str, attr: &str) -> String {
    format!("READ {} {} {} {}", dev, dir.as_str(), ch, attr)
}

pub(crate) fn cmd_write_channel_header(
    dev: &str,
    dir: Direction,
    ch: &str,
    attr: &str,
    bytelen: usize,
) -> String {
    format!("WRITE {} {} {} {} {}", dev, dir.as_str(), ch, attr, bytelen)
}

pub(crate) fn cmd_open(dev: &str, samples: usize, mask: u32, cyclic: bool) -> String {
    // The mask is one zero-padded 32-bit word wide (sufficient for <=32 channels,
    // which covers PlutoSDR and all common SDRs). A cyclic buffer repeats its
    // contents continuously, which is how a transmitter sustains a waveform.
    let suffix = if cyclic { " CYCLIC" } else { "" };
    format!("OPEN {} {} {:08x}{}", dev, samples, mask, suffix)
}

pub(crate) fn cmd_readbuf(dev: &str, nbytes: usize) -> String {
    format!("READBUF {} {}", dev, nbytes)
}

pub(crate) fn cmd_writebuf(dev: &str, nbytes: usize) -> String {
    format!("WRITEBUF {} {}", dev, nbytes)
}

// Debug attributes are device-level, not channel-level: the `DEBUG` token takes
// the direction slot and no channel name is supplied.

pub(crate) fn cmd_read_debug(dev: &str, attr: &str) -> String {
    format!("READ {} {} {}", dev, Direction::Debug.as_str(), attr)
}

pub(crate) fn cmd_write_debug_header(dev: &str, attr: &str, bytelen: usize) -> String {
    format!(
        "WRITE {} {} {} {}",
        dev,
        Direction::Debug.as_str(),
        attr,
        bytelen
    )
}

/// Map a negative iiod status (`-errno`) to an [`SdrError`].
pub(crate) fn iiod_errno(op: &str, status: i64) -> SdrError {
    SdrError::Hardware(format!("iiod {op} failed: errno {}", -status))
}

// --- Context XML parsing ------------------------------------------------------

/// A device discovered in the iiod context XML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IiodDeviceInfo {
    pub id: String,
    pub name: String,
    pub input_channels: usize,
    pub output_channels: usize,
}

/// Read an XML attribute value (`key="value"`) from a start-tag slice.
fn attr_value(tag: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Parse the iiod context XML into its device list, counting scan channels by
/// direction. Kept deliberately small and dependency-free (no XML crate).
pub fn parse_context_devices(xml: &str) -> Vec<IiodDeviceInfo> {
    let mut devices = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = xml[cursor..].find("<device ") {
        let dev_tag_start = cursor + rel;
        let tag_end = match xml[dev_tag_start..].find('>') {
            Some(e) => dev_tag_start + e,
            None => break,
        };
        let tag = &xml[dev_tag_start..tag_end];
        let id = attr_value(tag, "id").unwrap_or_default();
        let name = attr_value(tag, "name").unwrap_or_default();

        let body_start = tag_end + 1;
        let body_end = match xml[body_start..].find("</device>") {
            Some(e) => body_start + e,
            None => xml.len(),
        };
        let body = &xml[body_start..body_end];

        let (mut input, mut output) = (0usize, 0usize);
        let mut b = 0usize;
        while let Some(crel) = body[b..].find("<channel ") {
            let cstart = b + crel;
            let cend = match body[cstart..].find('>') {
                Some(e) => cstart + e,
                None => break,
            };
            match attr_value(&body[cstart..cend], "type").as_deref() {
                Some("input") => input += 1,
                Some("output") => output += 1,
                _ => {}
            }
            b = cend + 1;
        }

        devices.push(IiodDeviceInfo {
            id,
            name,
            input_channels: input,
            output_channels: output,
        });
        cursor = body_end + "</device>".len();
        if cursor >= xml.len() {
            break;
        }
    }
    devices
}

/// Convert normalized [`Complex32`] samples into interleaved little-endian int16
/// IQ bytes (`[i0,q0,i1,q1,...]`) for transmission.
///
/// Values are scaled by the TX full scale (32768) and **clamped** to the int16
/// range, so an out-of-range input saturates rather than wrapping around into
/// the opposite polarity.
pub fn complex32_to_iq_bytes(samples: &[Complex32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 4);
    for s in samples {
        for component in [s.re, s.im] {
            let scaled =
                (component * AD9361_TX_FULL_SCALE).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            out.extend_from_slice(&scaled.to_le_bytes());
        }
    }
    out
}

/// Convert interleaved little-endian int16 IQ bytes (`[i0,q0,i1,q1,...]`) into
/// normalized [`Complex32`] samples.
pub fn iq_bytes_to_complex32(bytes: &[u8]) -> Vec<Complex32> {
    let mut out = Vec::with_capacity(bytes.len() / 4);
    for pair in bytes.chunks_exact(4) {
        let i = i16::from_le_bytes([pair[0], pair[1]]) as f32 / AD9361_RX_FULL_SCALE;
        let q = i16::from_le_bytes([pair[2], pair[3]]) as f32 / AD9361_RX_FULL_SCALE;
        out.push(Complex32::new(i, q));
    }
    out
}

// --- The client ---------------------------------------------------------------

/// A connected iiod network client.
pub struct IiodClient {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}

impl IiodClient {
    /// Connect to an iiod endpoint with the default 5-second timeouts.
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        Self::connect_with_timeout(addr, Duration::from_secs(5))
    }

    /// Connect to an iiod endpoint, applying `timeout` to connect and to reads
    /// and writes. A short timeout is useful for best-effort device probing.
    pub fn connect_with_timeout(addr: SocketAddr, timeout: Duration) -> Result<Self> {
        let writer = TcpStream::connect_timeout(&addr, timeout)
            .map_err(|e| SdrError::Hardware(format!("iiod connect to {addr} failed: {e}")))?;
        writer.set_read_timeout(Some(timeout))?;
        writer.set_write_timeout(Some(timeout))?;
        let reader = BufReader::new(writer.try_clone()?);
        Ok(Self { writer, reader })
    }

    fn send(&mut self, line: &str) -> Result<()> {
        self.writer.write_all(line.as_bytes())?;
        self.writer.write_all(b"\r\n")?;
        self.writer.flush()?;
        Ok(())
    }

    fn read_line(&mut self) -> Result<String> {
        let mut s = String::new();
        let n = self.reader.read_line(&mut s)?;
        if n == 0 {
            return Err(SdrError::Hardware("iiod connection closed".to_string()));
        }
        Ok(s.trim_end_matches(['\r', '\n']).to_string())
    }

    /// Read a status/length line, returning the signed integer it carries.
    fn read_status(&mut self, op: &str) -> Result<i64> {
        let line = self.read_line()?;
        line.trim().parse::<i64>().map_err(|_| {
            SdrError::Protocol(format!("iiod {op}: expected integer status, got '{line}'"))
        })
    }

    /// Consume the single trailing newline the daemon appends after a text
    /// (`PRINT`/`READ`) payload. Raw buffer (`READBUF`) payloads have none.
    fn consume_trailing_newline(&mut self) -> Result<()> {
        let mut b = [0u8; 1];
        self.reader.read_exact(&mut b)?;
        if b[0] == b'\r' {
            self.reader.read_exact(&mut b)?;
        }
        Ok(())
    }

    /// Send `VERSION` and return the daemon version line.
    pub fn version(&mut self) -> Result<String> {
        self.send("VERSION")?;
        self.read_line()
    }

    /// Send `PRINT` and return the full context XML.
    pub fn print_context(&mut self) -> Result<String> {
        self.send("PRINT")?;
        let len = self.read_status("PRINT")?;
        if len < 0 {
            return Err(iiod_errno("PRINT", len));
        }
        let mut buf = vec![0u8; len as usize];
        self.reader.read_exact(&mut buf)?;
        self.consume_trailing_newline()?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }

    /// Read a channel attribute value (trailing NUL/whitespace stripped).
    pub fn read_channel_attr(
        &mut self,
        dev: &str,
        dir: Direction,
        ch: &str,
        attr: &str,
    ) -> Result<String> {
        self.send(&cmd_read_channel(dev, dir, ch, attr))?;
        let len = self.read_status("READ")?;
        if len < 0 {
            return Err(iiod_errno("READ", len));
        }
        let mut buf = vec![0u8; len as usize];
        self.reader.read_exact(&mut buf)?;
        self.consume_trailing_newline()?;
        let value = String::from_utf8_lossy(&buf);
        Ok(value.trim_end_matches(['\0', '\r', '\n', ' ']).to_string())
    }

    /// Write a channel attribute value (a trailing NUL is appended, matching libiio).
    pub fn write_channel_attr(
        &mut self,
        dev: &str,
        dir: Direction,
        ch: &str,
        attr: &str,
        value: &str,
    ) -> Result<()> {
        let mut data = value.as_bytes().to_vec();
        data.push(0);
        self.send(&cmd_write_channel_header(dev, dir, ch, attr, data.len()))?;
        self.writer.write_all(&data)?;
        self.writer.flush()?;
        let n = self.read_status("WRITE")?;
        if n < 0 {
            return Err(iiod_errno("WRITE", n));
        }
        Ok(())
    }

    /// Read a device debug attribute (e.g. `ad9361-phy`'s `loopback`).
    pub fn read_debug_attr(&mut self, dev: &str, attr: &str) -> Result<String> {
        self.send(&cmd_read_debug(dev, attr))?;
        let len = self.read_status("READ")?;
        if len < 0 {
            return Err(iiod_errno("READ", len));
        }
        let mut buf = vec![0u8; len as usize];
        self.reader.read_exact(&mut buf)?;
        self.consume_trailing_newline()?;
        let value = String::from_utf8_lossy(&buf);
        Ok(value.trim_end_matches(['\0', '\r', '\n', ' ']).to_string())
    }

    /// Write a device debug attribute (a trailing NUL is appended, matching libiio).
    pub fn write_debug_attr(&mut self, dev: &str, attr: &str, value: &str) -> Result<()> {
        let mut data = value.as_bytes().to_vec();
        data.push(0);
        self.send(&cmd_write_debug_header(dev, attr, data.len()))?;
        self.writer.write_all(&data)?;
        self.writer.flush()?;
        let n = self.read_status("WRITE")?;
        if n < 0 {
            return Err(iiod_errno("WRITE", n));
        }
        Ok(())
    }

    /// Open a buffer of `samples` samples with the given channel `mask`.
    pub fn open(&mut self, dev: &str, samples: usize, mask: u32) -> Result<()> {
        self.open_with(dev, samples, mask, false)
    }

    /// Open a buffer, optionally `cyclic`.
    ///
    /// A cyclic output buffer repeats its contents continuously until closed —
    /// required for a transmitter to sustain a waveform, since a one-shot buffer
    /// drains immediately.
    pub fn open_with(&mut self, dev: &str, samples: usize, mask: u32, cyclic: bool) -> Result<()> {
        self.send(&cmd_open(dev, samples, mask, cyclic))?;
        let r = self.read_status("OPEN")?;
        if r < 0 {
            return Err(iiod_errno("OPEN", r));
        }
        Ok(())
    }

    /// Close a previously opened capture buffer.
    pub fn close(&mut self, dev: &str) -> Result<()> {
        self.send(&format!("CLOSE {dev}"))?;
        let r = self.read_status("CLOSE")?;
        if r < 0 {
            return Err(iiod_errno("CLOSE", r));
        }
        Ok(())
    }

    /// Read `nbytes` of raw sample data from an open buffer.
    ///
    /// The `READBUF` response carries an `<nbytes>\n<mask>\n` header before the
    /// payload; the mask line is consumed here.
    pub fn read_buf(&mut self, dev: &str, nbytes: usize) -> Result<Vec<u8>> {
        self.send(&cmd_readbuf(dev, nbytes))?;
        let n = self.read_status("READBUF")?;
        if n < 0 {
            return Err(iiod_errno("READBUF", n));
        }
        // The daemon echoes the channel mask on its own line before the payload.
        let _mask = self.read_line()?;
        let mut buf = vec![0u8; n as usize];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Write raw sample data to an open output buffer.
    ///
    /// Sends `WRITEBUF <dev> <nbytes>` followed by the payload and returns the
    /// byte count the daemon accepted.
    ///
    /// # Safety of use
    ///
    /// On a real radio this drives the transmitter. Callers are responsible for
    /// ensuring transmission is intended and lawful — see
    /// [`crate::pluto::PlutoSdr::enter_loopback_test_mode`] for the
    /// internal-loopback verification path.
    pub fn write_buf(&mut self, dev: &str, data: &[u8]) -> Result<usize> {
        self.send(&cmd_writebuf(dev, data.len()))?;
        // `WRITEBUF` is a two-phase exchange (verified live against iiod 0.21):
        // the daemon first acknowledges the header with a status line, and only
        // then accepts the payload. Reading a single status here would both
        // report 0 bytes written and desynchronize the connection.
        let ready = self.read_status("WRITEBUF")?;
        if ready < 0 {
            return Err(iiod_errno("WRITEBUF", ready));
        }
        self.writer.write_all(data)?;
        self.writer.flush()?;
        let n = self.read_status("WRITEBUF")?;
        if n < 0 {
            return Err(iiod_errno("WRITEBUF", n));
        }
        Ok(n as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iiod_command_framing() {
        assert_eq!(
            cmd_read_channel("iio:device0", Direction::Output, "altvoltage0", "frequency"),
            "READ iio:device0 OUTPUT altvoltage0 frequency"
        );
        assert_eq!(
            cmd_open("iio:device3", 1024, 0x3, false),
            "OPEN iio:device3 1024 00000003"
        );
        assert_eq!(
            cmd_open("iio:device2", 1024, 0x3, true),
            "OPEN iio:device2 1024 00000003 CYCLIC",
            "a cyclic TX buffer repeats until closed"
        );
        assert_eq!(cmd_readbuf("iio:device3", 4096), "READBUF iio:device3 4096");
    }

    #[test]
    fn test_iiod_debug_direction_token() {
        // Debug attributes are device-level: the DEBUG token replaces the
        // direction and no channel name appears.
        assert_eq!(
            cmd_read_debug("iio:device0", "loopback"),
            "READ iio:device0 DEBUG loopback"
        );
        // "1" plus a trailing NUL = 2 bytes.
        assert_eq!(
            cmd_write_debug_header("iio:device0", "loopback", 2),
            "WRITE iio:device0 DEBUG loopback 2"
        );
        assert_eq!(Direction::Debug.as_str(), "DEBUG");
    }

    #[test]
    fn test_iiod_writebuf_framing() {
        assert_eq!(
            cmd_writebuf("iio:device2", 4096),
            "WRITEBUF iio:device2 4096"
        );
    }

    #[test]
    fn test_iiod_writebuf_error_surfaced() {
        // A negative WRITEBUF status maps to a hardware error, never a panic.
        match iiod_errno("WRITEBUF", -22) {
            SdrError::Hardware(msg) => assert!(msg.contains("errno 22"), "{msg}"),
            other => panic!("expected Hardware error, got {other:?}"),
        }
    }

    #[test]
    fn test_iiod_complex32_to_iq_bytes() {
        // TX uses the full 16-bit scale (32768), unlike RX's 12-bit 2048.
        let samples = [Complex32::new(1.0, -1.0), Complex32::new(0.0, 0.5)];
        let bytes = complex32_to_iq_bytes(&samples);
        assert_eq!(bytes.len(), 8, "2 samples x I/Q x 2 bytes");

        let i0 = i16::from_le_bytes([bytes[0], bytes[1]]);
        let q0 = i16::from_le_bytes([bytes[2], bytes[3]]);
        let i1 = i16::from_le_bytes([bytes[4], bytes[5]]);
        let q1 = i16::from_le_bytes([bytes[6], bytes[7]]);
        assert_eq!(i0, i16::MAX, "1.0 saturates at +32767");
        assert_eq!(q0, i16::MIN, "-1.0 maps to -32768");
        assert_eq!(i1, 0);
        assert_eq!(q1, 16384, "0.5 * 32768");
    }

    #[test]
    fn test_iiod_tx_scale_clamps() {
        // Out-of-range input must saturate, not wrap into the opposite polarity.
        let samples = [Complex32::new(5.0, -5.0)];
        let bytes = complex32_to_iq_bytes(&samples);
        let i = i16::from_le_bytes([bytes[0], bytes[1]]);
        let q = i16::from_le_bytes([bytes[2], bytes[3]]);
        assert_eq!(i, i16::MAX);
        assert_eq!(q, i16::MIN);
    }

    #[test]
    fn test_iiod_attr_write_framing() {
        // Value "40.000000" is 9 bytes + a trailing NUL = 10.
        assert_eq!(
            cmd_write_channel_header(
                "iio:device0",
                Direction::Input,
                "voltage0",
                "hardwaregain",
                10
            ),
            "WRITE iio:device0 INPUT voltage0 hardwaregain 10"
        );
    }

    #[test]
    fn test_iiod_error_surfaced() {
        // -EINVAL from OPEN maps to a hardware error rather than a panic.
        let err = iiod_errno("OPEN", -22);
        match err {
            SdrError::Hardware(msg) => assert!(msg.contains("errno 22"), "{msg}"),
            other => panic!("expected Hardware error, got {other:?}"),
        }
    }

    #[test]
    fn test_iiod_iq_int16_to_complex32() {
        // i0=2048 (=> 1.0), q0=-2048 (=> -1.0), i1=0, q1=1024 (=> 0.5)
        let bytes = [
            0x00, 0x08, // 2048 LE
            0x00, 0xF8, // -2048 LE
            0x00, 0x00, // 0
            0x00, 0x04, // 1024 LE
        ];
        let iq = iq_bytes_to_complex32(&bytes);
        assert_eq!(iq.len(), 2);
        assert!((iq[0].re - 1.0).abs() < 1e-6);
        assert!((iq[0].im + 1.0).abs() < 1e-6);
        assert!((iq[1].re).abs() < 1e-6);
        assert!((iq[1].im - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_iiod_context_parse() {
        let xml = include_str!("../tests/fixtures/pluto_ctx.xml");
        let devs = parse_context_devices(xml);
        let phy = devs
            .iter()
            .find(|d| d.name == "ad9361-phy")
            .expect("ad9361-phy present");
        assert!(phy.input_channels > 0 && phy.output_channels > 0);

        let rx = devs
            .iter()
            .find(|d| d.name == "cf-ad9361-lpc")
            .expect("cf-ad9361-lpc present");
        assert_eq!(rx.input_channels, 4, "RX device has 4 scan channels");
        assert_eq!(rx.id, "iio:device3");
    }
}
