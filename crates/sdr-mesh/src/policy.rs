//! Dual-mode compliance gate.
//!
//! Encryption is legal on ISM bands and prohibited on amateur allocations, so
//! the mesh runs in one of two modes chosen per frequency. This gate reuses the
//! project's regulatory database (INT-0005) to decide, before every
//! transmission, whether an encrypted payload may go out, whether to fall back
//! to open (unencrypted) operation, or whether to refuse entirely.

use sdr_core::compliance::{ComplianceResult, Jurisdiction, RegulatoryDatabase};

/// The two transmission modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxMode {
    /// Encrypted payloads permitted (ISM bands).
    Encrypted,
    /// Unencrypted only (amateur bands, AREDN-style).
    Open,
}

/// Outcome of a compliance evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Transmission permitted in the given mode on the matched band.
    Allow {
        mode: TxMode,
        band_name: String,
        citation: String,
    },
    /// Transmission refused; `reasons` carries the regulator's rationale.
    Refuse { reasons: Vec<String> },
}

impl Decision {
    /// Whether the decision permits transmission.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Decision::Allow { .. })
    }

    /// The permitted mode, if allowed.
    pub fn mode(&self) -> Option<TxMode> {
        match self {
            Decision::Allow { mode, .. } => Some(*mode),
            Decision::Refuse { .. } => None,
        }
    }
}

/// The mesh transmission-compliance gate.
pub struct MeshPolicy;

impl MeshPolicy {
    /// Evaluate whether a transmission is permitted, consulting the regulatory
    /// database. When `want_encrypted` is set, the band must permit encryption
    /// (ISM) or the transmission is refused — the mesh never silently sends
    /// encrypted payloads where prohibited.
    pub fn evaluate(
        jurisdiction: Jurisdiction,
        freq_hz: u64,
        power_dbm: f32,
        want_encrypted: bool,
    ) -> Decision {
        match RegulatoryDatabase::check_compliance(jurisdiction, freq_hz, power_dbm, want_encrypted)
        {
            ComplianceResult::Compliant {
                band_name,
                citation,
                ..
            } => Decision::Allow {
                mode: if want_encrypted {
                    TxMode::Encrypted
                } else {
                    TxMode::Open
                },
                band_name: band_name.to_string(),
                citation: citation.to_string(),
            },
            ComplianceResult::NonCompliant { reasons } => Decision::Refuse { reasons },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // US 915 MHz ISM permits encryption; US 2m/70cm amateur prohibit it.
    const ISM_915: u64 = 915_000_000;
    const AMATEUR_2M: u64 = 145_000_000;
    const UNCATALOGED: u64 = 50_000_000;

    #[test]
    fn test_gate_encrypted_ism_allowed() {
        let d = MeshPolicy::evaluate(Jurisdiction::US, ISM_915, 20.0, true);
        assert_eq!(d.mode(), Some(TxMode::Encrypted), "got {d:?}");
    }

    #[test]
    fn test_gate_encrypted_refused() {
        let d = MeshPolicy::evaluate(Jurisdiction::US, AMATEUR_2M, 20.0, true);
        match d {
            Decision::Refuse { reasons } => {
                assert!(!reasons.is_empty(), "refusal should carry a reason");
                assert!(
                    reasons
                        .iter()
                        .any(|r| r.to_uppercase().contains("ENCRYPTION")),
                    "reason should cite encryption: {reasons:?}"
                );
            }
            other => panic!("expected Refuse on amateur band with encryption, got {other:?}"),
        }
    }

    #[test]
    fn test_gate_open_allowed() {
        let d = MeshPolicy::evaluate(Jurisdiction::US, AMATEUR_2M, 20.0, false);
        assert_eq!(d.mode(), Some(TxMode::Open), "got {d:?}");
    }

    #[test]
    fn test_gate_uncataloged_refused() {
        let d = MeshPolicy::evaluate(Jurisdiction::US, UNCATALOGED, 20.0, false);
        assert!(
            !d.is_allowed(),
            "uncataloged frequency should be refused: {d:?}"
        );
    }
}
