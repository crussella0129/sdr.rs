//! Two-node mesh loopback: IP datagrams transit end-to-end through KISS framing
//! over the ARQ link layer, and the compliance gate is enforced at the node.
//! Deterministic, in-memory — no radio, no threads.

use sdr_core::compliance::Jurisdiction;
use sdr_mesh::{LoopbackLink, MeshNode};

// A cataloged ISM band (encryption permitted) and a cataloged amateur band
// (encryption prohibited), both US.
const ISM_915: u64 = 915_000_000;
const AMATEUR_2M: u64 = 145_000_000;

#[test]
fn test_two_node_datagram_roundtrip() {
    let (link_a, link_b) = LoopbackLink::pair(1, 2);
    // Both nodes on US 915 MHz ISM, so encrypted payloads are permitted.
    let mut node_a = MeshNode::new(link_a, Jurisdiction::US, ISM_915, 20.0);
    let mut node_b = MeshNode::new(link_b, Jurisdiction::US, ISM_915, 20.0);

    // A synthetic IP-ish datagram containing bytes that must be KISS-escaped.
    let datagram = vec![0x45, 0x00, 0x00, 0x28, 0xC0, 0xDB, 0x11, 0x22];

    node_a
        .send(&datagram, /*want_encrypted=*/ true)
        .expect("encrypted send on ISM should be allowed");

    let received = node_b
        .recv()
        .expect("recv")
        .expect("a datagram should arrive");
    assert_eq!(
        received, datagram,
        "B must receive the exact datagram A sent"
    );

    // Reverse direction, unencrypted.
    let reply = b"pong".to_vec();
    node_b.send(&reply, false).expect("open send");
    assert_eq!(
        node_a.recv().expect("recv").as_deref(),
        Some(reply.as_slice())
    );
}

#[test]
fn test_node_refuses_encrypted_on_amateur() {
    let (link_a, link_b) = LoopbackLink::pair(1, 2);
    // Both nodes on US 2m amateur — encryption is prohibited here.
    let mut node_a = MeshNode::new(link_a, Jurisdiction::US, AMATEUR_2M, 20.0);
    let mut node_b = MeshNode::new(link_b, Jurisdiction::US, AMATEUR_2M, 20.0);

    let result = node_a.send(b"secret", /*want_encrypted=*/ true);
    assert!(
        result.is_err(),
        "encrypted TX on an amateur band must be refused"
    );

    // No frame was emitted, so the peer receives nothing.
    assert_eq!(node_b.recv().expect("recv"), None);

    // The same node may still transmit in the clear (open mode) on this band.
    node_a
        .send(b"cq de node-a", false)
        .expect("open send on amateur should be allowed");
    assert_eq!(
        node_b.recv().expect("recv").as_deref(),
        Some(b"cq de node-a".as_slice())
    );
}
