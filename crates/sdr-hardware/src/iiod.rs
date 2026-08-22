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
//! - `CLOSE <dev>\r\n` → `<0>\n`.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Result, SdrError};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// Standard libiio network daemon (`iiod`) TCP port.
pub const IIOD_PORT: u16 = 30431;

/// Full-scale divisor for AD9361 12-bit signed IQ samples (`S12/16` format).
const AD9361_RX_FULL_SCALE: f32 = 2048.0;

/// IIO channel direction token used in `READ`/`WRITE` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
}

impl Direction {
    fn as_str(self) -> &'static str {
        match self {
            Direction::Input => "INPUT",
            Direction::Output => "OUTPUT",
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

pub(crate) fn cmd_open(dev: &str, samples: usize, mask: u32) -> String {
    // The mask is one zero-padded 32-bit word wide (sufficient for <=32 channels,
    // which covers PlutoSDR and all common SDRs).
    format!("OPEN {} {} {:08x}", dev, samples, mask)
}

pub(crate) fn cmd_readbuf(dev: &str, nbytes: usize) -> String {
    format!("READBUF {} {}", dev, nbytes)
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
    /// Connect to an iiod endpoint, applying connect/read/write timeouts.
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let writer = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
            .map_err(|e| SdrError::Hardware(format!("iiod connect to {addr} failed: {e}")))?;
        writer.set_read_timeout(Some(Duration::from_secs(5)))?;
        writer.set_write_timeout(Some(Duration::from_secs(5)))?;
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

    /// Open a capture buffer of `samples` samples with the given channel `mask`.
    pub fn open(&mut self, dev: &str, samples: usize, mask: u32) -> Result<()> {
        self.send(&cmd_open(dev, samples, mask))?;
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
            cmd_open("iio:device3", 1024, 0x3),
            "OPEN iio:device3 1024 00000003"
        );
        assert_eq!(cmd_readbuf("iio:device3", 4096), "READBUF iio:device3 4096");
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
