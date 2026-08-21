# Sprint 1 Research Report

## Intents Reviewed
- [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) — created; relevance: jurisdictional regulatory database and legal compliance advisor for encrypted data and power limits; current state: proposed
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — created; relevance: packet radio transceiver, ARQ link layer, and bidirectional SSH stream tunneling; current state: proposed

## 1. Sprint Goal
Design and implement the Jurisdictional RF Regulatory Band Advisor (`sdr_compliance`) and the Bidirectional Packet Radio Transceiver & SSH Tunneling subsystem. The engine enables users to identify legally permissible transmission bands based on their geographic region (US, EU, UK, AU, and Global) with explicit verification of encryption legality (differentiating license-exempt ISM bands from amateur radio bands where encryption is prohibited by law), while providing a robust packet radio link layer with ARQ retransmission, GFSK/FSK modulation, TX hardware driver streaming, and an OpenSSH proxy tunnel interface.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| crates/sdr-core/src/traits.rs | high | Core Source, Sink, Block traits to extend with TX capabilities |
| crates/sdr-dsp/src/fir.rs | high | Pulse shaping filters (Gaussian / Root Raised Cosine) for TX modulation |
| crates/sdr-dsp/src/nco.rs | high | NCO frequency translation for TX upconversion |
| crates/sdr-hardware/src/driver.rs | high | SdrDriver trait to support TX buffer streaming (`write_samples`, `start_tx`, `stop_tx`) |
| crates/sdr-hardware/src/pluto.rs | high | PlutoSDR AD9361 TX channels and buffer streaming |
| crates/sdr-hardware/src/mock.rs | high | MockSdr to support bidirectional loopback testing |
| crates/sdr-demod/src/fsk.rs | medium | FSK discriminator to pair with FSK/GFSK packet modulator |
| crates/sdr-protocols/src/lib.rs | high | Host for new packet framing and ARQ link layer |
| crates/sdr-cli/src/main.rs | high | CLI interface to add `bands` advisor and `tunnel` subcommands |

## 3. External Sources
- [FCC Part 15 Subpart C (47 CFR 15.247 / 15.249)](https://www.ecfr.gov/current/title-47/chapter-I/subchapter-A/part-15) — US license-exempt ISM regulations for 902–928 MHz, 2.400–2.4835 GHz, and 5.725–5.875 GHz (EIRP limits, FHSS/DSSS rules; encryption permitted).
- [FCC Part 97 (47 CFR 97.113(a)(4))](https://www.ecfr.gov/current/title-47/chapter-I/subchapter-D/part-97) — Prohibits transmissions in the Amateur Radio Service having "messages encoded for the purpose of obscuring their meaning" (prohibiting SSH/TLS encryption on Ham bands like 2m/70cm).
- [CEPT ERC Recommendation 70-03 & ETSI EN 300 220](https://docdb.cept.org/document/705) — European Short Range Devices (SRD) regulations for 863–870 MHz, 433.05–434.79 MHz (ERP limits, duty cycles 0.1% to 10%, channel spacing; encryption permitted).
- [OpenSSH ProxyCommand Specification & RFC 4251](https://tools.ietf.org/html/rfc4251) — Bidirectional byte-stream tunneling over standard I/O pipes.

## 4. Architectural Analysis & Design

### 4.1 Regulatory Compliance & Band Advisor (`sdr_compliance`)
- **Jurisdictions Supported**:
  - `US` (FCC): 902–928 MHz (ISM Part 15.247: up to 1W / 30 dBm conducted, 36 dBm EIRP with FHSS/DSSS, encryption allowed), 2.4 GHz ISM, 5.8 GHz ISM, Amateur 2m (144–148 MHz, encryption prohibited), Amateur 70cm (420–450 MHz, encryption prohibited), Amateur 33cm (902–928 MHz amateur allocation, encryption prohibited).
  - `EU` / `UK` (ETSI / CEPT Rec 70-03 / Ofcom): 863–870 MHz (SRD 868.0–868.6 MHz 25 mW ERP / 1% duty cycle; 869.4–869.65 MHz 500 mW ERP / 10% duty cycle, encryption allowed), 433.05–434.79 MHz (10 mW ERP / 10% duty cycle), 2.4 GHz (100 mW EIRP), Amateur 2m (144–146 MHz, encryption prohibited), Amateur 70cm (430–440 MHz, encryption prohibited).
  - `AU` (ACMA): 915–928 MHz LIPD class license (up to 1W EIRP, encryption allowed), 433 MHz LIPD, 2.4 GHz.
  - `Global`: 2.400–2.4835 GHz ISM and 5.725–5.875 GHz ISM.
- **Advisory Engine**:
  - Query parameters: `jurisdiction`, `frequency_hz`, `power_dbm`, `bandwidth_hz`, `encrypted: bool`.
  - Validates compliance and returns clear warnings if encrypted payloads are planned for Amateur radio bands.

### 4.2 Packet Radio Protocol & ARQ Link Layer (`sdr-protocols::packet`)
- **Frame Structure**:
  - Preamble: Alternating `0xAA 0xAA 0xAA 0xAA` (32 bits) for bit clock synchronization.
  - Sync Word: `0xD3 0x91 0xD3 0x91` (32 bits) for byte framing alignment.
  - Header: Source Address (1 byte), Destination Address (1 byte), Sequence Number (2 bytes), Flags / Type (`DATA`, `ACK`, `NACK`, `HEARTBEAT`), Payload Length (2 bytes).
  - Payload: 0 to 1024 bytes.
  - CRC-32 Checksum: 4 bytes standard IEEE 802.3 polynomial over header and payload.
- **ARQ Protocol**:
  - Stop-and-Wait ARQ with exponential backoff retransmissions for lossy RF channels.
  - Deduplication of retransmitted sequence numbers at receiver.

### 4.3 Modulation & Transmit Engine (`sdr-demod`, `sdr-hardware`)
- **GFSK / 2-FSK Modulator**:
  - Continuous-phase frequency synthesizer with Gaussian pulse shaping FIR filter.
  - Generates complex baseband IQ bursts ready for direct transmission.
- **TX Driver Extension**:
  - Extends `SdrDriver` with `start_tx()`, `stop_tx()`, `write_samples()`.
  - Implements TX path in `MockSdr` (with simulated loopback channel with configurable packet drop rate) and `PlutoSdr`.

### 4.4 SSH Stream Tunneling Bridge (`sdr-cli tunnel`)
- Runs as an OpenSSH ProxyCommand (`ssh -o ProxyCommand="sdr-cli tunnel --peer <addr> --freq <hz>" user@radio-node`).
- Transports stdin/stdout byte streams transparently across packet radio frames with ARQ reliability.

## 5. Risks, Unknowns, Dependencies
- **Risk:** High latency in half-duplex packet switching during SSH initial key exchange.
  - *Mitigation:* Buffer SSH burst packets into maximum-sized MTU frames (512–1024 bytes) and use fast ACK turnaround.
- **Unknown:** Variations in local EIRP vs ERP definition across different national regulators.
  - *Mitigation:* Document conversion factors (EIRP = ERP + 2.15 dB) and display both in CLI output.
- **Dependency:** Pure Rust standard networking and math crates without external C dependencies.

## 6. Recommended Approach
- Primary: Implement `sdr-compliance` module within the workspace, `sdr-protocols::packet` for the reliable packet engine and GFSK modulator, extend `sdr-hardware` with TX streaming, and add `bands` and `tunnel` subcommands to `sdr-cli`.
- Rationale: Fully modular, cross-platform, clean adherence to Project Book intents, and 100% testable in CI via simulated mock loopback.

## 7. Artifacts
- `crates/sdr-compliance` (or `crates/sdr-core/src/compliance.rs` / `crates/sdr-protocols/src/packet.rs`)
