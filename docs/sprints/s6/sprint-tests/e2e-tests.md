# Sprint 6 End-to-End Tests

- **Tested head:** `2821e3fc480b7eaf6fde937dfc12252ff8ea8680`
- **Status:** possible — live hardware E2E performed by the agent; CI stays hardware-free (4 `#[ignore]`d tests).

## Live hardware
`hw_verify_mesh_datagram_over_radio`, run against the physical Pluto+ at
`192.168.2.1:30431` under the standing internal-loopback configuration
(maximum attenuation, DDS silenced):

```
sent 10 bytes through the Pluto+ (internal loopback, max attenuation); recovered Some(10)
```

**PASS** — the identical datagram was recovered through the real radio using the
single timing-recovered demodulation pass, with the candidate-phase loop gone.
The known risk (a cyclic buffer's wrap discontinuity briefly unlocking the loop)
did not materialise: capturing well beyond the frame length leaves a complete
frame clear of the wrap. Device state (`loopback`, TX gain, DDS) independently
confirmed restored afterward.

### What this test does and does not show
It shows the swap did not break integration with a real device — real DMA, real
timing, real sample rates. It **cannot** evidence drift tolerance: the internal
loopback shares one clock between transmitter and receiver, so there is no
offset to track. Drift is proven in CI, where an offset can be injected
deliberately. Neither test is claimed to do the other's job.

## Not-yet-possible (named unlockers)
- **Behaviour under channel noise / BER characterization** → needs a real channel; the internal loopback is noiseless. Carried forward.
- **Two separate radios exchanging datagrams** → needs a second radio and over-the-air operation (T-108), which requires explicit go-ahead. This sprint removes the *shared-clock* blocker to that, but does not attempt it.
- **Multi-hop routing** → Phase B (T-107), `tun`/babeld.
