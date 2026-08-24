//! Dual-mode compliance gate.
//!
//! Encryption is legal on ISM bands and prohibited on amateur allocations, so
//! the mesh runs in one of two modes chosen per frequency. This gate reuses the
//! project's regulatory database (INT-0005) to decide, before every
//! transmission, whether an encrypted payload may go out, whether to fall back
//! to open (unencrypted) operation, or whether to refuse entirely.

use crate::node::MeshInterface;
use sdr_core::compliance::{ComplianceResult, RegulatoryDatabase, TransmissionPlan};
use sdr_core::traits::{Result, SdrError};

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
    /// Evaluate a complete transmission plan against the regulatory catalog.
    /// The mesh never silently changes an invalid plan or downgrades its
    /// encryption mode.
    pub fn evaluate(plan: &TransmissionPlan) -> Decision {
        match RegulatoryDatabase::check_compliance(plan) {
            ComplianceResult::Compliant {
                band_name,
                citation,
                ..
            } => Decision::Allow {
                mode: if plan.encrypted {
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

/// A transmission interface that re-evaluates its complete plan before every
/// datagram, preventing later callers from bypassing the compliance gate.
pub struct PolicyCheckedInterface<I> {
    iface: I,
    plan: TransmissionPlan,
}

impl<I> PolicyCheckedInterface<I> {
    pub fn new(iface: I, plan: TransmissionPlan) -> Self {
        Self { iface, plan }
    }
}

impl<I: MeshInterface> MeshInterface for PolicyCheckedInterface<I> {
    fn send_datagram(&mut self, datagram: &[u8]) -> Result<()> {
        match MeshPolicy::evaluate(&self.plan) {
            Decision::Allow { .. } => self.iface.send_datagram(datagram),
            Decision::Refuse { reasons } => {
                log::warn!("mesh TX refused by compliance gate: {}", reasons.join("; "));
                Err(SdrError::Config(format!(
                    "mesh transmission refused by compliance gate: {}",
                    reasons.join("; ")
                )))
            }
        }
    }

    fn recv_datagram(&mut self) -> Result<Option<Vec<u8>>> {
        self.iface.recv_datagram()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdr_core::compliance::Jurisdiction;
    use std::cell::Cell;
    use std::rc::Rc;

    // US 915 MHz ISM permits encryption; US 2m/70cm amateur prohibit it.
    const ISM_915: u64 = 915_000_000;
    const AMATEUR_2M: u64 = 145_000_000;
    const UNCATALOGED: u64 = 50_000_000;

    fn plan(center_frequency_hz: u64, encrypted: bool) -> TransmissionPlan {
        TransmissionPlan {
            jurisdiction: Jurisdiction::US,
            center_frequency_hz: center_frequency_hz as f64,
            occupied_bandwidth_hz: 100_000.0,
            eirp_dbm: 20.0,
            duty_cycle_pct: 100.0,
            encrypted,
        }
    }

    #[test]
    fn test_gate_encrypted_ism_allowed() {
        let d = MeshPolicy::evaluate(&plan(ISM_915, true));
        assert_eq!(d.mode(), Some(TxMode::Encrypted), "got {d:?}");
    }

    #[test]
    fn test_gate_encrypted_refused() {
        let d = MeshPolicy::evaluate(&plan(AMATEUR_2M, true));
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
        let d = MeshPolicy::evaluate(&plan(AMATEUR_2M, false));
        assert_eq!(d.mode(), Some(TxMode::Open), "got {d:?}");
    }

    #[test]
    fn test_gate_uncataloged_refused() {
        let d = MeshPolicy::evaluate(&plan(UNCATALOGED, false));
        assert!(
            !d.is_allowed(),
            "uncataloged frequency should be refused: {d:?}"
        );
    }

    struct CountingInterface {
        sends: Rc<Cell<usize>>,
    }

    impl MeshInterface for CountingInterface {
        fn send_datagram(&mut self, _datagram: &[u8]) -> Result<()> {
            self.sends.set(self.sends.get() + 1);
            Ok(())
        }

        fn recv_datagram(&mut self) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }
    }

    #[test]
    fn test_policy_checked_interface_refusal_emits_no_frame() {
        let sends = Rc::new(Cell::new(0));
        let mut iface = PolicyCheckedInterface::new(
            CountingInterface {
                sends: Rc::clone(&sends),
            },
            plan(AMATEUR_2M, true),
        );

        assert!(iface.send_datagram(b"encrypted payload").is_err());
        assert_eq!(sends.get(), 0, "refused policy must not call inner send");
    }
}
