//! Two-node mesh loopback: IP datagrams transit end-to-end through KISS framing
//! over the ARQ link layer, and the compliance gate is enforced at the node.
//! Deterministic, in-memory — no radio, no threads.

use sdr_core::compliance::{Jurisdiction, TransmissionPlan};
use sdr_core::traits::Result;
use sdr_mesh::{LoopbackLink, MeshInterface, MeshNode};
use std::cell::Cell;
use std::rc::Rc;

// A cataloged ISM band (encryption permitted) and a cataloged amateur band
// (encryption prohibited), both US.
const ISM_915: u64 = 915_000_000;
const AMATEUR_2M: u64 = 145_000_000;

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
fn test_two_node_datagram_roundtrip() {
    let (link_a, link_b) = LoopbackLink::pair(1, 2);
    // Both nodes on US 915 MHz ISM, so encrypted payloads are permitted.
    let mut node_a = MeshNode::new(link_a, plan(ISM_915, true));
    let mut node_b = MeshNode::new(link_b, plan(ISM_915, true));

    // A synthetic IP-ish datagram containing bytes that must be KISS-escaped.
    let datagram = vec![0x45, 0x00, 0x00, 0x28, 0xC0, 0xDB, 0x11, 0x22];

    node_a
        .send(&datagram)
        .expect("encrypted send on ISM should be allowed");

    let received = node_b
        .recv()
        .expect("recv")
        .expect("a datagram should arrive");
    assert_eq!(
        received, datagram,
        "B must receive the exact datagram A sent"
    );

    // Reverse direction.
    let reply = b"pong".to_vec();
    node_b.send(&reply).expect("reverse send");
    assert_eq!(
        node_a.recv().expect("recv").as_deref(),
        Some(reply.as_slice())
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
fn test_node_policy_refusal_emits_no_frame() {
    let sends = Rc::new(Cell::new(0));
    let mut node = MeshNode::new(
        CountingInterface {
            sends: Rc::clone(&sends),
        },
        plan(AMATEUR_2M, true),
    );

    let result = node.send(b"secret");
    assert!(
        result.is_err(),
        "encrypted TX on an amateur band must be refused"
    );
    assert_eq!(sends.get(), 0, "a refusal must not reach the interface");
}

#[test]
fn test_node_allows_open_amateur_roundtrip() {
    let (link_a, link_b) = LoopbackLink::pair(1, 2);
    let mut node_a = MeshNode::new(link_a, plan(AMATEUR_2M, false));
    let mut node_b = MeshNode::new(link_b, plan(AMATEUR_2M, false));

    node_a
        .send(b"cq de node-a")
        .expect("open send on amateur should be allowed");
    assert_eq!(
        node_b.recv().expect("recv").as_deref(),
        Some(b"cq de node-a".as_slice())
    );
}
