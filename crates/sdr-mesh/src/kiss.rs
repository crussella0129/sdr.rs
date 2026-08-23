//! KISS framing — the amateur-radio standard for delimiting datagrams on a
//! byte-oriented link. Used here to preserve IP datagram boundaries over the
//! packet-radio byte stream.
//!
//! A frame is `FEND … FEND`; any `FEND`/`FESC` byte inside the payload is
//! escaped so the delimiters stay unambiguous.

/// Frame delimiter.
pub const FEND: u8 = 0xC0;
/// Frame escape.
pub const FESC: u8 = 0xDB;
/// Transposed frame end (follows `FESC` to mean a literal `FEND`).
pub const TFEND: u8 = 0xDC;
/// Transposed frame escape (follows `FESC` to mean a literal `FESC`).
pub const TFESC: u8 = 0xDD;

/// Encode a datagram as a single KISS frame: `FEND`, byte-stuffed payload, `FEND`.
pub fn encode(datagram: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(datagram.len() + 2);
    out.push(FEND);
    for &b in datagram {
        match b {
            FEND => out.extend_from_slice(&[FESC, TFEND]),
            FESC => out.extend_from_slice(&[FESC, TFESC]),
            _ => out.push(b),
        }
    }
    out.push(FEND);
    out
}

/// Streaming KISS decoder. Feed arbitrary byte chunks; it returns each complete
/// datagram once and retains any partial frame across calls.
#[derive(Debug, Default)]
pub struct KissDecoder {
    buf: Vec<u8>,
    escaped: bool,
}

impl KissDecoder {
    /// Create an empty decoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed `bytes`, returning any datagrams that completed. Partial input is
    /// buffered for a later call.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut done = Vec::new();
        for &b in bytes {
            if self.escaped {
                match b {
                    TFEND => self.buf.push(FEND),
                    TFESC => self.buf.push(FESC),
                    // Not a valid transposition; keep the literal byte rather than panic.
                    other => self.buf.push(other),
                }
                self.escaped = false;
                continue;
            }
            match b {
                // FEND both closes the current frame and opens the next; an
                // empty frame (FEND FEND) carries no datagram and is ignored.
                FEND => {
                    if !self.buf.is_empty() {
                        done.push(std::mem::take(&mut self.buf));
                    }
                }
                FESC => self.escaped = true,
                _ => self.buf.push(b),
            }
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kiss_roundtrip() {
        let datagram = b"hello mesh \x45\x00\x00\x28"; // arbitrary bytes incl. an IPv4-ish header start
        let framed = encode(datagram);
        assert_eq!(framed.first(), Some(&FEND));
        assert_eq!(framed.last(), Some(&FEND));

        let mut dec = KissDecoder::new();
        let out = dec.push(&framed);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], datagram);
    }

    #[test]
    fn test_kiss_byte_stuffing() {
        // Payload full of the special bytes must survive intact.
        let datagram = [FEND, FESC, 0x00, FEND, TFEND, FESC, TFESC];
        let framed = encode(&datagram);
        // The raw frame body must not contain a bare FEND/FESC except delimiters.
        assert_eq!(
            framed.iter().filter(|&&b| b == FEND).count(),
            2,
            "only the two delimiters"
        );

        let mut dec = KissDecoder::new();
        let out = dec.push(&framed);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], datagram);
    }

    #[test]
    fn test_kiss_stream_reassembly() {
        let a = b"first".to_vec();
        let b = b"second".to_vec();
        let mut stream = encode(&a);
        stream.extend(encode(&b));
        // Split the concatenated frames at an awkward boundary, leave a trailing partial.
        let partial = encode(b"third");
        stream.extend_from_slice(&partial[..3]); // only the start of the third frame

        let mut dec = KissDecoder::new();
        let split = 4;
        let mut out = dec.push(&stream[..split]);
        out.extend(dec.push(&stream[split..]));

        assert_eq!(
            out.len(),
            2,
            "exactly the two complete datagrams, not the partial"
        );
        assert_eq!(out[0], a);
        assert_eq!(out[1], b);

        // Completing the third frame yields it and nothing duplicated.
        let rest = dec.push(&partial[3..]);
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0], b"third");
    }
}
