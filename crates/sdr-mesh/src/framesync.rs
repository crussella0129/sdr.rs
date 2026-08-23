//! Bit-level frame synchronization.
//!
//! A demodulator emits a continuous bit stream with no notion of byte
//! boundaries: the receiver may start mid-byte, so packing bits into bytes from
//! index zero generally yields garbage. This module finds the packet sync word
//! at **bit** granularity and repacks from there, producing bytes that begin
//! exactly at the sync word — the form [`sdr_protocols::packet::PacketFramer`]
//! already scans for.

use sdr_protocols::packet::SYNC_WORD;

/// Expand bytes into bits, MSB first (the order the modulator transmits).
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<u8> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for &b in bytes {
        for pos in (0..8).rev() {
            bits.push((b >> pos) & 1);
        }
    }
    bits
}

/// Pack bits into bytes, MSB first. A trailing partial byte is discarded.
pub fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bits.len() / 8);
    for chunk in bits.chunks_exact(8) {
        let mut byte = 0u8;
        for &bit in chunk {
            byte = (byte << 1) | (bit & 1);
        }
        out.push(byte);
    }
    out
}

/// Locate the sync word in `bits` and return the bytes starting at it.
///
/// Searches at every bit position, so an arbitrary bit offset introduced by the
/// demodulator is recovered. Returns `None` when the sync word is absent.
pub fn sync_to_frame(bits: &[u8]) -> Option<Vec<u8>> {
    let pattern = bytes_to_bits(&SYNC_WORD);
    if bits.len() < pattern.len() {
        return None;
    }
    let start =
        (0..=bits.len() - pattern.len()).find(|&i| bits[i..i + pattern.len()] == pattern[..])?;
    Some(bits_to_bytes(&bits[start..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framesync_packs_msb_first() {
        // 0b1010_0101 = 0xA5
        let bits = [1, 0, 1, 0, 0, 1, 0, 1];
        assert_eq!(bits_to_bytes(&bits), vec![0xA5]);
        assert_eq!(bytes_to_bits(&[0xA5]), bits);
    }

    #[test]
    fn test_framesync_finds_sync_at_bit_offsets() {
        let frame: Vec<u8> = SYNC_WORD
            .iter()
            .copied()
            .chain([0x01, 0x02, 0x03, 0x04])
            .collect();

        // Prepend 0..8 filler bits so the sync word lands at every bit offset.
        for offset in 0..8usize {
            let mut bits = vec![0u8; offset];
            bits.extend(bytes_to_bits(&frame));

            let recovered = sync_to_frame(&bits)
                .unwrap_or_else(|| panic!("sync word not found at bit offset {offset}"));
            assert_eq!(
                &recovered[..SYNC_WORD.len()],
                &SYNC_WORD[..],
                "frame must begin at the sync word (offset {offset})"
            );
            assert_eq!(
                &recovered[SYNC_WORD.len()..SYNC_WORD.len() + 4],
                &[0x01, 0x02, 0x03, 0x04],
                "payload must follow intact (offset {offset})"
            );
        }
    }

    #[test]
    fn test_framesync_absent_returns_none() {
        // A stream of zeros contains no sync word.
        let bits = vec![0u8; 256];
        assert!(sync_to_frame(&bits).is_none());
        // Too short to contain one.
        assert!(sync_to_frame(&[1, 0, 1]).is_none());
    }
}
