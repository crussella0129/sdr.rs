# Sprint 0 Research Report

## Intents Reviewed
- [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) — created; relevance: foundational DSP streaming and SIMD architecture; current state: proposed
- [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) — created; relevance: hardware abstraction and PlutoSDR / IIO Gigabit Ethernet / USB integration; current state: proposed
- [INT-0003](../../../intents/INT-0003-modulation-demodulation.md) — created; relevance: universal analog and digital modulation/demodulation blocks; current state: proposed
- [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) — created; relevance: wireless protocol decoders (LoRa, ADS-B, APRS) and real-time spectrum analysis; current state: proposed

## 1. Sprint Goal
Conduct a comprehensive architectural and algorithmic research ingestion across 16 foundational SDR reference codebases and hardware platforms (GNU Radio, SDR++, Pluto+ / AD936x IIO, cuda-oxide, GNSS-SDR, CuPy, Mini Radio Telescope, RadioLib, HackRF, URH, Radio ML, Hamlib, EHT Imaging, gr-lora, RadioSniffer, and the 50-things SDR ecosystem). Synthesize the findings into the core architecture, workspace crate structure, streaming contracts, hardware driver abstraction, and DSP pipelines for `sdr.rs`, establishing the Project Book intents and execution foundation for Sprint 0.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| README.md | high | Declares project mission and the 16 core reference codebases for initial research ingestion |
| .gitignore | high | Configured for Rust builds and Sprint Loops v2 helper tracking |
| docs/work/remote-profile.md | medium | Defines GitHub provider, main/dev branch topology, and human-approve merge policy |

## 3. External Sources
- [GNU Radio Framework](https://github.com/gnuradio/gnuradio) — Thread-per-block scheduler, circular buffer VM memory mapping, synchronous and asynchronous stream tags (PMT), and extensive DSP filter/digital demodulator libraries.
- [SDR++ Modular Receiver](https://github.com/AlexandreRouma/SDRPlusPlus) — High-efficiency C++ SDR receiver architecture, dynamic plugin manager, SIMD-accelerated DSP, multi-VFO extraction from master digitizer bandwidth, and network streaming sinks.
- [50 Things with SDR](https://blinry.org/50-things-with-sdr/) — Broad capability roadmap across broadcast, aviation (ADS-B/ACARS), maritime (AIS), satellites (NOAA APT/Meteor), IoT (433/915 MHz, LoRa), public safety (P25/DMR), and radio astronomy.
- [Pluto+ SDR Setup & Architecture](https://www.sdrstore.eu/pluto-plus-sdr-setup-guide-sdrangel-gnu-radio-ethernet/) — Hardware details for AD9363/AD9364 transceiver (70 MHz - 6 GHz, up to 56 MHz RF BW, 12-bit ADC/DAC), Zynq-7010 FPGA, Gigabit Ethernet PHY, and libiio network streaming.
- [cuda-oxide](https://github.com/NVlabs/cuda-oxide) — Idiomatic safe Rust compilation directly to NVPTX / CUDA PTX kernels via LLVM backend, enabling zero-overhead GPU-accelerated DSP in Rust.
- [GNSS-SDR](https://github.com/gnss-sdr/gnss-sdr) — Multi-constellation GNSS software receiver architecture (GPS, Galileo, GLONASS, BeiDou) featuring parallel FFT acquisition, carrier/code tracking loops (PLL/DLL), telemetry decoding, and PVT solver.
- [CuPy GPU Array Engine](https://github.com/cupy/cupy) — High-throughput GPU array processing, custom kernel JIT, memory pool allocators preventing allocation latency, and cuFFT integration.
- [MiniRadioTelescope](https://github.com/UPennEoR/MiniRadioTelescope) — Radio astronomy spectrometer engine for the 21-cm (1420.405 MHz) neutral hydrogen line, baseline subtraction, long FFT integration, and dual-axis rotator control.
- [RadioLib](https://github.com/jgromes/RadioLib) — Universal wireless communication library covering physical-layer transceiver drivers (SX127x/SX126x/SX128x LoRa, CC1101, RFM69) and modulations (FSK, GFSK, OOK, LoRa, AX.25, RTTY, POCSAG).
- [HackRF One](https://github.com/greatscottgadgets/hackrf) — Half-duplex transceiver firmware, C host library (`libhackrf`), asynchronous USB 2.0 streaming via libusb, and high-speed wideband frequency sweep mode (`hackrf_sweep`).
- [Universal Radio Hacker (URH)](https://github.com/jopohl/urh) — Complete RF reverse-engineering suite: IQ recording, DC/IQ balance correction, automatic modulation detection (ASK/FSK/PSK), adaptive bit slicing, differential decoding, and packet fuzzing.
- [Radio (AnalysisCenter)](https://github.com/analysiscenter/radio) — Deep learning framework for RF signal processing: synthetic RF channel augmentation (AWGN, fading, CFO/SCO), STFT spectrogram generation, and automatic modulation recognition (AMR).
- [Hamlib](https://github.com/Hamlib/Hamlib) — Unified CAT transceiver control and antenna rotator interface across 400+ radio models, providing TCP rigctl protocol (port 4532) for external software integration.
- [eht-imaging](https://github.com/achael/eht-imaging) — Radio interferometry and VLBI image reconstruction (Event Horizon Telescope), closure phases, visibility modeling, and Regularized Maximum Likelihood (RML) imaging.
- [gr-lora](https://github.com/rpp0/gr-lora) — Full physical-layer LoRa Chirp Spread Spectrum (CSS) receiver: preamble correlation, CFO estimation, de-chirping, FFT symbol extraction, de-interleaving, Hamming FEC decoding, and CRC verification.
- [RadioSniffer](https://github.com/AlexMalov/RadioSniffer) — Autonomous wideband spectrum monitor: CFAR energy detection, polyphase filterbank channelization, SNR-based squelch, and multi-channel audio/data streaming.

## 4. Architectural Analysis & Component Mapping

### 4.1 DSP Engine & Streaming Architecture
- **Inspiration**: GNU Radio circular buffer model + SDR++ SIMD vectorization.
- **`sdr.rs` Design**:
  - `sdr-core`: Defines zero-copy sample buffers (`SampleBuffer<T>`), stream tags (`StreamTag` with key, value, sample offset), and composable processing traits (`Source`, `Sink`, `Block`, `Stream`).
  - `sdr-dsp`: SIMD-accelerated math kernels (using `std::simd` and auto-vectorization), windowed FIR filter generator (Hamming, Hann, Blackman-Harris, Kaiser), polyphase rational resampler, NCO / DDS frequency translation, Hilbert transform for analytic IQ generation, Costas loop for phase recovery, and Gardner / Mueller & Müller clock recovery for symbol timing.

### 4.2 Hardware Abstraction Layer & PlutoSDR Integration
- **Inspiration**: Pluto+ IIO network backend + HackRF USB streaming + SoapySDR unified interface.
- **`sdr.rs` Design**:
  - `sdr-hardware`: `SdrDriver` trait providing uniform configuration (`set_frequency`, `set_sample_rate`, `set_gain`, `set_bandwidth`) and asynchronous RX/TX stream acquisition.
  - Native `PlutoSdr` driver connecting via TCP/IP (`ip:192.168.1.10`) to Linux IIO subsystem (`ad9361-phy` control interface and `cf-ad9361-lpc` continuous streaming buffers) and USB backend.
  - `FileDriver`: Support for SigMF standard (JSON metadata + raw dataset), RIFF WAV RF64, and raw binary IQ streams (`cf32`, `cs16`, `cu8`).
  - Mock drivers for deterministic, hardware-free automated testing.

### 4.3 Demodulation and Wireless Protocols
- **Inspiration**: RadioLib + URH + gr-lora + SDR++.
- **`sdr.rs` Design**:
  - `sdr-demod`:
    - Analog: WFM (with 19 kHz stereo pilot PLL and RDS carrier extraction), NFM (with audio low-pass and CTCSS tone squelch), AM (envelope and synchronous), SSB (USB/LSB via phasing/Weaver), and CW beat tone detector.
    - Digital: OOK/ASK, FSK/GFSK (quadrature discriminator + Manchester/NRZ slicer), BPSK/QPSK (Costas loop + Gray demapping).
  - `sdr-protocols`:
    - LoRa CSS Decoder: Preamble cross-correlation, fractional frequency offset estimation, conjugate down-chirp FFT symbol demodulation, diagonal de-interleaving, Hamming FEC (CR 4/5 - 4/8), and payload CRC-16 check.
    - ADS-B Mode S Decoder: 1090 MHz pulse position demodulation, CRC-24 parity check, aircraft altitude, position (CPR decoding), and callsign decoding.
    - APRS / AX.25: 1200 baud Bell 202 AFSK demodulation, HDLC flag framing, and packet parsing.

### 4.4 Spectrum Analysis & External Interoperability
- **Inspiration**: RadioSniffer + MiniRadioTelescope + Hamlib.
- **`sdr.rs` Design**:
  - `sdr-spectrum`: High-speed windowed FFT power spectrum estimation, peak detector, CFAR adaptive noise floor estimation, and waterfall spectrogram generation.
  - `sdr-server`: Embedded Rigctl server implementing standard Hamlib TCP protocol (port 4532) for bidirectional frequency/mode synchronization with external astronomy and tracking tools (GPredict, WSJT-X).

## 5. Risks, Unknowns, Dependencies
- **Risk:** High-throughput streaming at > 20 MSPS in pure Rust could encounter memory copy bottlenecks if buffers are reallocated dynamically.
  - *Mitigation:* Employ pre-allocated ring buffers and zero-copy slice views across processing stages.
- **Unknown:** Physical Pluto+ hardware availability and network latency over Gigabit Ethernet across different operating systems (Windows vs Linux).
  - *Mitigation:* Build a robust mock driver framework and unit tests with synthetic IQ data, enabling full test coverage without connected physical hardware.
- **Dependency:** Reliance on standard Rust numerical crates (`num-complex`, `rustfft`) while keeping foreign C dependencies strictly optional.

## 6. Recommended Approach
- **Primary:** Establish a modular Rust Cargo workspace:
  - `crates/sdr-core`: Sample types, stream tags, buffer abstractions, and core processing traits.
  - `crates/sdr-dsp`: Filter design, SIMD kernels, resamplers, NCO oscillators, and synchronization loops.
  - `crates/sdr-hardware`: `SdrDriver` trait, PlutoSDR IIO client, file drivers (SigMF/WAV/Raw), and mock hardware.
  - `crates/sdr-demod`: Analog (WFM, NFM, AM, SSB, CW) and digital (OOK, FSK, PSK) demodulators.
  - `crates/sdr-protocols`: LoRa, ADS-B Mode S, and APRS/AX.25 packet decoders.
  - `crates/sdr-spectrum`: FFT spectrum analyzer, CFAR detection, and Hamlib Rigctl TCP server.
  - `crates/sdr-cli`: Command-line tools for recording, playing, demodulating, and inspecting RF streams.
- **Alternative considered:** Monolithic single crate. Rejected to allow downstream users and embedded deployments to depend only on needed components (e.g. DSP without GUI/hardware).
- **Rationale:** Cargo workspace architecture ensures modularity, fast incremental compilation, clean API boundaries, and clear alignment with Book intent chapters.

## 7. Artifacts
- `Cargo.toml` — Workspace manifest configuring member crates and shared dependencies.
- `crates/sdr-core/src/lib.rs` — Core streaming and sample definitions.

## Budget Override
The survey includes 16 external references corresponding directly to the explicit research ingestion catalog mandated in `README.md` (spanning GNU Radio, SDR++, PlutoSDR IIO, cuda-oxide, GNSS-SDR, CuPy, MiniRadioTelescope, RadioLib, HackRF, URH, Radio ML, Hamlib, EHT Imaging, gr-lora, RadioSniffer, and 50-things SDR). Comprehensive ingestion of these 16 reference architectures is necessary to establish the unified architecture, crate topology, and intent structure for the `sdr.rs` suite.
