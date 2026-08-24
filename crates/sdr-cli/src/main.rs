//! # sdr-cli
//!
//! Command-line interface for the `sdr.rs` cross-platform SDR suite.

use clap::{Parser, Subcommand};
use sdr_core::compliance::{ComplianceResult, Jurisdiction, RegulatoryDatabase, TransmissionPlan};
use sdr_core::sample::Complex32;
use sdr_core::traits::SdrError;
use sdr_demod::{AmDemod, DeEmphasis, FskDemod, NfmDemod, SsbDemod, SsbMode, WfmDemod};
use sdr_dsp::window::WindowType;
use sdr_hardware::driver::SdrDriver;
use sdr_hardware::mock::{MockSdr, MockSignal};
use sdr_hardware::pluto::PlutoSdr;
use sdr_hardware::sigmf::{SigMfReader, SigMfWriter};
use sdr_hardware::wav::{read_iq_wav, write_iq_wav};
use sdr_mesh::policy::{Decision, MeshPolicy, PolicyCheckedInterface};
use sdr_mesh::{RadioLink, RadioParams, StreamBridge, BROADCAST_ADDR};
use sdr_spectrum::cfar::CaCfarDetector;
use sdr_spectrum::fft::SpectrumAnalyzer;
use sdr_spectrum::rigctl::{RigState, RigctlHandler};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sdr-cli")]
#[command(about = "A comprehensive cross-platform SDR suite in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect an SDR dataset (SigMF or WAV IQ file)
    Info {
        /// Path to dataset (.sigmf-meta, .sigmf-data, or .wav)
        file: PathBuf,
    },
    /// Record IQ samples to disk (SigMF or WAV format)
    Record {
        /// Output file path (.sigmf-data or .wav)
        #[arg(short, long)]
        output: PathBuf,

        /// Center frequency in Hz (e.g. 915000000)
        #[arg(short, long, default_value_t = 915.0e6)]
        freq: f64,

        /// Sample rate in samples/sec (e.g. 2000000)
        #[arg(short, long, default_value_t = 2.0e6)]
        rate: f64,

        /// Number of samples to record
        #[arg(short, long, default_value_t = 100_000)]
        samples: usize,

        /// Driver to use (mock, pluto)
        #[arg(short, long, default_value = "mock")]
        driver: String,
    },
    /// Demodulate an IQ recording to audio or bitstream
    Demod {
        /// Input IQ file
        #[arg(short, long)]
        input: PathBuf,

        /// Demodulation mode (wfm, nfm, am, ssb, fsk)
        #[arg(short, long, default_value = "wfm")]
        mode: String,

        /// Output file path (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Perform FFT spectral analysis and CFAR peak detection on IQ capture
    Spectrum {
        /// Input IQ file
        #[arg(short, long)]
        input: PathBuf,

        /// FFT size (e.g. 1024, 2048, 4096)
        #[arg(long, default_value_t = 1024)]
        fft_size: usize,
    },
    /// Query jurisdictional RF regulations, legal frequency bands, and transmission compliance
    Bands {
        /// Geographic jurisdiction (US, EU, UK, AU, Global)
        #[arg(short, long, default_value = "US")]
        jurisdiction: Jurisdiction,

        /// Require legal support for encrypted payloads (such as SSH / TLS)
        #[arg(short, long, default_value_t = false)]
        encrypted: bool,

        /// Specific frequency in Hz to check compliance (optional)
        #[arg(long)]
        check_freq: Option<f64>,

        /// Occupied transmit bandwidth in Hz for a compliance check
        #[arg(long, requires = "check_freq")]
        occupied_bandwidth_hz: Option<f64>,

        /// Transmit duty cycle in percent for a compliance check
        #[arg(long, requires = "check_freq")]
        duty_cycle_pct: Option<f64>,

        /// Transmit EIRP in dBm for a compliance check
        #[arg(short, long, visible_alias = "eirp-dbm", default_value_t = 20.0)]
        power: f64,
    },
    /// Packet Radio Bidirectional Tunnel bridge for "SSH over Radio"
    Tunnel {
        /// Peer radio address (0-255)
        #[arg(short, long, default_value_t = 2)]
        peer: u8,

        /// Local radio address (0-255)
        #[arg(short, long, default_value_t = 1)]
        local_addr: u8,

        /// Center frequency in Hz (e.g. 915000000 for US ISM, 868000000 for EU SRD)
        #[arg(short, long, default_value_t = 915.0e6)]
        freq: f64,

        /// Sample rate in samples/sec
        #[arg(short, long, default_value_t = 1.0e6)]
        rate: f64,

        /// Jurisdiction for regulatory compliance verification
        #[arg(short, long, default_value = "US")]
        jurisdiction: Jurisdiction,

        /// Operator-stated effective isotropic radiated power in dBm
        #[arg(long)]
        eirp_dbm: f64,

        /// Pipe stdin/stdout over the link — the OpenSSH ProxyCommand contract.
        /// Use as: ssh -o ProxyCommand="sdr-cli tunnel --stdio ..." host
        #[arg(long)]
        stdio: bool,

        /// Accept one TCP connection on this port and pipe it over the link
        #[arg(long)]
        listen: Option<u16>,

        /// Driver to use (mock, pluto, or an ip:/usb: iiod URI)
        #[arg(short, long, default_value = "mock")]
        driver: String,

        /// Bytes of stream payload per datagram
        #[arg(long, default_value_t = sdr_mesh::DEFAULT_MTU)]
        mtu: usize,
    },
    /// Start Hamlib Rigctl TCP server for external radio control
    Rigctl {
        /// TCP port to bind (default: 4532)
        #[arg(short, long, default_value_t = 4532)]
        port: u16,
    },
    /// Enumerate available SDR devices (mock, and any reachable PlutoSDR)
    Devices,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemodMode {
    Wfm,
    Nfm,
    Am,
    Ssb,
    Fsk,
}

impl DemodMode {
    fn parse(value: &str) -> Result<Self, SdrError> {
        match value.to_ascii_lowercase().as_str() {
            "wfm" => Ok(Self::Wfm),
            "nfm" => Ok(Self::Nfm),
            "am" => Ok(Self::Am),
            "ssb" => Ok(Self::Ssb),
            "fsk" => Ok(Self::Fsk),
            other => Err(SdrError::Config(format!(
                "Unknown demodulation mode '{other}'. Supported: wfm, nfm, am, ssb, fsk"
            ))),
        }
    }

    fn is_audio(self) -> bool {
        !matches!(self, Self::Fsk)
    }
}

enum DemodOutput {
    Audio(Vec<f32>),
    Bits(Vec<u8>),
}

fn validate_demod_output(
    mode: DemodMode,
    output: Option<&std::path::Path>,
) -> Result<(), SdrError> {
    let Some(path) = output else {
        return Ok(());
    };
    if !mode.is_audio() {
        return Err(SdrError::Config(
            "FSK file output is unsupported; omit --output to inspect the recovered bit count"
                .to_string(),
        ));
    }
    let is_wav = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"));
    if !is_wav {
        return Err(SdrError::Config(
            "audio demodulation output must use a .wav path".to_string(),
        ));
    }
    Ok(())
}

fn wav_sample_rate(sample_rate_hz: f32) -> Result<u32, SdrError> {
    const U32_EXCLUSIVE_UPPER_BOUND: f32 = 4_294_967_296.0;
    let rounded = sample_rate_hz.round();
    if !sample_rate_hz.is_finite()
        || sample_rate_hz <= 0.0
        || !(1.0..U32_EXCLUSIVE_UPPER_BOUND).contains(&rounded)
    {
        return Err(SdrError::Config(format!(
            "WAV sample rate must be finite and representable as a positive u32 (got {sample_rate_hz:?} Hz)"
        )));
    }
    Ok(rounded as u32)
}

fn write_audio_wav<W: std::io::Write + std::io::Seek>(
    writer: W,
    sample_rate: u32,
    audio: &[f32],
) -> hound::Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut wav = hound::WavWriter::new(writer, spec)?;
    for &sample in audio {
        let pcm = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        wav.write_sample(pcm)?;
    }
    wav.finalize()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { file } => {
            println!("=== sdr.rs Dataset Inspector ===");
            println!("File: {}", file.display());

            let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext == "sigmf-meta"
                || ext == "sigmf-data"
                || file.with_extension("sigmf-meta").exists()
            {
                let reader = SigMfReader::open(&file)?;
                println!("Format: SigMF (Signal Metadata Format)");
                println!("Datatype: {}", reader.metadata.global.datatype);
                println!(
                    "Sample Rate: {:.3} MSPS",
                    reader.metadata.global.sample_rate / 1e6
                );
                if let Some(cap) = reader.metadata.captures.first() {
                    println!("Center Frequency: {:.3} MHz", cap.frequency / 1e6);
                }
                println!(
                    "Recorder: {}",
                    reader
                        .metadata
                        .global
                        .recorder
                        .as_deref()
                        .unwrap_or("unknown")
                );
            } else if ext == "wav" {
                let (rate, samples) = read_iq_wav(&file)?;
                println!("Format: RIFF WAV IQ");
                println!("Sample Rate: {} Hz", rate);
                println!("Total Samples: {}", samples.len());
                println!(
                    "Duration: {:.2} seconds",
                    samples.len() as f64 / rate as f64
                );
            } else {
                println!(
                    "Unknown file extension '{}'. Supported: .sigmf-meta, .sigmf-data, .wav",
                    ext
                );
            }
        }
        Commands::Record {
            output,
            freq,
            rate,
            samples,
            driver,
        } => {
            println!("=== sdr.rs IQ Recorder ===");
            println!(
                "Driver: {}, Target Freq: {:.3} MHz, Rate: {:.3} MSPS, Samples: {}",
                driver,
                freq / 1e6,
                rate / 1e6,
                samples
            );

            let mut buffer = vec![Complex32::default(); samples];
            let driver_kind = driver.to_lowercase();
            if driver_kind == "mock" {
                let mut sdr = MockSdr::new(rate, freq);
                sdr.set_signal(MockSignal::Tone {
                    offset_hz: 10000.0,
                    amplitude: 0.8,
                });
                sdr.start_rx()?;
                sdr.read_samples(&mut buffer)?;
                sdr.stop_rx()?;
            } else {
                // "pluto" uses the default network endpoint; an explicit
                // "ip:"/"usb:" value is passed through as the iiod URI.
                let mut sdr = if driver_kind == "pluto" {
                    PlutoSdr::default_network()?
                } else {
                    PlutoSdr::new(&driver)?
                };
                sdr.set_sample_rate(0, rate)?;
                sdr.set_frequency(0, freq)?;
                sdr.start_rx()?;
                let n = sdr.read_samples(&mut buffer)?;
                buffer.truncate(n);
                sdr.stop_rx()?;
                sdr.teardown()?;
            }

            let ext = output.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext == "wav" {
                write_iq_wav(&output, rate as u32, &buffer)?;
                println!(
                    "Saved {} samples to WAV IQ: {}",
                    buffer.len(),
                    output.display()
                );
            } else {
                let mut writer = SigMfWriter::create(&output, rate, freq)?;
                writer.write_samples(&buffer)?;
                writer.close()?;
                println!(
                    "Saved {} samples to SigMF archive: {}",
                    buffer.len(),
                    output.display()
                );
            }
        }
        Commands::Demod {
            input,
            mode,
            output,
        } => {
            let demod_mode = DemodMode::parse(&mode)?;
            validate_demod_output(demod_mode, output.as_deref())?;

            println!("=== sdr.rs Demodulator ===");
            println!("Input: {}, Mode: {}", input.display(), mode);

            let (sample_rate, samples) = if input.extension().is_some_and(|e| e == "wav") {
                let (rate, s) = read_iq_wav(&input)?;
                (rate as f32, s)
            } else {
                let mut reader = SigMfReader::open(&input)?;
                let rate = reader.metadata.global.sample_rate as f32;
                let mut buf = vec![Complex32::default(); 100_000];
                let n = reader.read_samples(&mut buf)?;
                buf.truncate(n);
                (rate, buf)
            };

            println!(
                "Loaded {} IQ samples at {:.1} kHz",
                samples.len(),
                sample_rate / 1e3
            );

            let demodulated = match demod_mode {
                DemodMode::Wfm => {
                    let mut demod = WfmDemod::new(sample_rate, 75000.0, DeEmphasis::Eu50us);
                    let mut out = Vec::new();
                    demod.demod_block(&samples, &mut out);
                    DemodOutput::Audio(out)
                }
                DemodMode::Nfm => {
                    let mut demod = NfmDemod::new(sample_rate, 5000.0, -40.0);
                    let mut out = Vec::new();
                    for &s in &samples {
                        out.push(demod.demod_sample(s));
                    }
                    DemodOutput::Audio(out)
                }
                DemodMode::Am => {
                    let mut demod = AmDemod::new();
                    let mut out = Vec::new();
                    for &s in &samples {
                        out.push(demod.demod_sample(s));
                    }
                    DemodOutput::Audio(out)
                }
                DemodMode::Ssb => {
                    let mut demod = SsbDemod::new(SsbMode::Usb);
                    let mut out = Vec::new();
                    for &s in &samples {
                        out.push(demod.demod_sample(s));
                    }
                    DemodOutput::Audio(out)
                }
                DemodMode::Fsk => {
                    let mut demod = FskDemod::new(sample_rate, 5000.0, 10);
                    let mut bits = Vec::new();
                    demod.demod_bits(&samples, &mut bits);
                    DemodOutput::Bits(bits)
                }
            };

            if let Some(out_path) = output {
                match demodulated {
                    DemodOutput::Audio(audio) => {
                        let output_sample_rate = wav_sample_rate(sample_rate)?;
                        let file = std::fs::File::create(&out_path)?;
                        write_audio_wav(file, output_sample_rate, &audio)?;
                        println!(
                            "Exported {} audio samples to {}",
                            audio.len(),
                            out_path.display()
                        );
                    }
                    DemodOutput::Bits(_) => {
                        return Err(
                            SdrError::Config("FSK file output is unsupported".to_string()).into(),
                        );
                    }
                }
            } else {
                match demodulated {
                    DemodOutput::Audio(audio) => println!(
                        "Demodulation completed successfully ({} audio samples produced)",
                        audio.len()
                    ),
                    DemodOutput::Bits(bits) => println!(
                        "Demodulation completed successfully ({} bits produced)",
                        bits.len()
                    ),
                }
            }
        }
        Commands::Spectrum { input, fft_size } => {
            println!("=== sdr.rs Spectrum Analyzer ===");
            let (_rate, samples) = if input.extension().is_some_and(|e| e == "wav") {
                read_iq_wav(&input)?
            } else {
                let mut reader = SigMfReader::open(&input)?;
                let mut buf = vec![Complex32::default(); fft_size];
                let n = reader.read_samples(&mut buf)?;
                buf.truncate(n);
                (reader.metadata.global.sample_rate as u32, buf)
            };

            let mut analyzer = SpectrumAnalyzer::new(fft_size, WindowType::BlackmanHarris, 1.0);
            let spectrum = analyzer.process(&samples);
            let (peak_bin, peak_pwr) = SpectrumAnalyzer::find_peak_in(spectrum);
            println!("FFT Size: {}", fft_size);
            println!("Strongest Peak: Bin {} ({:.1} dBFS)", peak_bin, peak_pwr);

            let cfar = CaCfarDetector::new(4, 16, 15.0);
            let detections = cfar.detect(spectrum);
            println!(
                "CFAR Detections ({} active emitters found):",
                detections.len()
            );
            for (bin, pwr, thresh) in detections.iter().take(5) {
                println!(
                    "  - Bin {}: {:.1} dBFS (Noise Floor Threshold: {:.1} dBFS)",
                    bin, pwr, thresh
                );
            }
        }
        Commands::Bands {
            jurisdiction,
            encrypted,
            check_freq,
            occupied_bandwidth_hz,
            duty_cycle_pct,
            power,
        } => {
            println!("=== sdr.rs RF Regulatory Advisor ===");
            println!("Selected Jurisdiction: {}", jurisdiction.as_str());
            println!(
                "Encryption Support Required: {}",
                if encrypted {
                    "YES (e.g. SSH / TLS / AES)"
                } else {
                    "NO (Open telemetry / voice)"
                }
            );

            if let Some(freq) = check_freq {
                let occupied_bandwidth_hz = occupied_bandwidth_hz
                    .ok_or("--occupied-bandwidth-hz is required with --check-freq")?;
                let duty_cycle_pct =
                    duty_cycle_pct.ok_or("--duty-cycle-pct is required with --check-freq")?;
                let plan = TransmissionPlan {
                    jurisdiction,
                    center_frequency_hz: freq,
                    occupied_bandwidth_hz,
                    eirp_dbm: power,
                    duty_cycle_pct,
                    encrypted,
                };
                println!(
                    "\n--- Compliance Verification for {:.3} MHz ---",
                    freq / 1e6
                );
                let result = RegulatoryDatabase::check_compliance(&plan);
                match result {
                    ComplianceResult::Compliant {
                        band_name,
                        citation,
                        warnings,
                    } => {
                        println!("Result: COMPLIANT [PASS]");
                        println!("Matched Band: {}", band_name);
                        println!("Regulatory Authority: {}", citation);
                        for w in warnings {
                            println!("  [Advisory Warning] {}", w);
                        }
                    }
                    ComplianceResult::NonCompliant { reasons } => {
                        println!("Result: NON-COMPLIANT [VIOLATION]");
                        for r in reasons {
                            println!("  [Violation Reason] {}", r);
                        }
                    }
                }
            } else {
                let bands = RegulatoryDatabase::query_recommended_bands(jurisdiction, encrypted);
                println!(
                    "\nRecommended Legal Frequency Bands ({} matches):",
                    bands.len()
                );
                for b in bands {
                    println!("\n* {}", b.name);
                    println!(
                        "  - Frequency Range: {:.3} MHz - {:.3} MHz",
                        b.start_freq_hz as f64 / 1e6,
                        b.end_freq_hz as f64 / 1e6
                    );
                    println!(
                        "  - Max Transmit Power: {:.1} dBm ({:.1} W)",
                        b.max_power_dbm,
                        10.0f32.powf((b.max_power_dbm - 30.0) / 10.0)
                    );
                    if let Some(dc) = b.max_duty_cycle_pct {
                        println!("  - Duty Cycle Limit: {:.1}%", dc);
                    }
                    println!(
                        "  - Encrypted Payloads: {}",
                        if b.encryption_permitted {
                            "PERMITTED (License-Free)"
                        } else {
                            "PROHIBITED BY LAW"
                        }
                    );
                    println!("  - Legal Citation: {}", b.citation);
                }
            }
        }
        Commands::Tunnel {
            peer,
            local_addr,
            freq,
            rate: requested_rate,
            jurisdiction,
            eirp_dbm,
            stdio,
            listen,
            driver,
            mtu,
        } => {
            let modem_defaults = RadioParams::default();
            let (plan, params) = tunnel_transmission_plan(
                jurisdiction,
                freq,
                eirp_dbm,
                requested_rate,
                modem_defaults.deviation_hz,
                modem_defaults.samples_per_symbol,
            )?;
            let rate = f64::from(params.sample_rate);

            // Status goes to stderr: in --stdio mode stdout carries the tunnelled
            // stream and must not be polluted.
            eprintln!("=== sdr.rs SSH over Radio Tunnel Bridge ===");
            eprintln!("Local Station: 0x{local_addr:02X}, Peer Station: 0x{peer:02X}");
            if rate != requested_rate {
                eprintln!(
                    "Requested sample rate {requested_rate:.6} Hz normalized to the modem/driver rate {rate:.6} Hz"
                );
            }
            eprintln!(
                "Frequency: {:.3} MHz, Rate: {:.3} MSPS, Jurisdiction: {}, MTU: {mtu}",
                freq / 1e6,
                rate / 1e6,
                jurisdiction.as_str()
            );
            eprintln!("Operator-stated EIRP: {eirp_dbm:.1} dBm");

            // Evaluate the complete plan before selecting or constructing a
            // driver. A refusal is terminal; encrypted traffic is never sent
            // after a warning-only result.
            match MeshPolicy::evaluate(&plan) {
                Decision::Allow {
                    band_name,
                    citation,
                    ..
                } => eprintln!("Regulatory Status: COMPLIANT ({band_name}) - {citation}"),
                Decision::Refuse { reasons } => {
                    let message = format!(
                        "transmission refused by compliance gate: {}",
                        reasons.join("; ")
                    );
                    eprintln!("Regulatory Status: REFUSED");
                    for r in reasons {
                        eprintln!("  - {r}");
                    }
                    return Err(
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, message).into(),
                    );
                }
            }

            if !stdio && listen.is_none() {
                eprintln!(
                    "No mode selected. Use --stdio (OpenSSH ProxyCommand) or --listen <port>."
                );
                return Ok(());
            }

            let kind = driver.to_lowercase();
            if kind == "mock" {
                let mut sdr = MockSdr::new(rate, freq);
                sdr.enable_loopback();
                sdr.start_tx()?;
                sdr.start_rx()?;
                // The mock driver echoes what it transmits, so a frame addressed
                // to a distinct peer would be filtered out on return. Address
                // broadcast: this is a self-echo loopback, not a two-station link.
                run_tunnel(
                    sdr,
                    TunnelEndpoint {
                        local_addr,
                        peer: BROADCAST_ADDR,
                        mtu,
                        stdio,
                        listen,
                    },
                    params,
                    plan,
                )?;
            } else {
                let mut sdr = if kind == "pluto" {
                    PlutoSdr::default_network()?
                } else {
                    PlutoSdr::new(&driver)?
                };
                sdr.set_sample_rate(0, rate)?;
                sdr.set_frequency(0, freq)?;
                sdr.set_tx_frequency(freq)?;
                sdr.start_tx()?;
                sdr.start_rx()?;
                run_tunnel(
                    sdr,
                    TunnelEndpoint {
                        local_addr,
                        peer,
                        mtu,
                        stdio,
                        listen,
                    },
                    params,
                    plan,
                )?;
            }
        }
        Commands::Rigctl { port } => {
            println!("=== sdr.rs Hamlib Rigctl Server ===");
            run_rigctl_server(port).await?;
        }
        Commands::Devices => {
            println!("=== sdr.rs Device Enumeration ===");
            let devices = sdr_hardware::list_devices();
            for d in &devices {
                println!(
                    "- {} [{}] rx_channels={} tx_channels={}",
                    d.name, d.uri, d.rx_channels, d.tx_channels
                );
            }
            println!("{} device(s) available", devices.len());
        }
    }

    Ok(())
}

/// Poll interval for the tunnel pumps — bounded so neither direction starves
/// and the loop does not spin a core.
const TUNNEL_POLL: std::time::Duration = std::time::Duration::from_millis(5);
/// Bytes taken from the link per pump iteration.
const TUNNEL_READ_MAX: usize = 4096;

/// Construct the complete tunnel policy input from the modem parameters.
/// Binary FSK occupied bandwidth uses Carson's estimate. The tunnel has no
/// transmit limiter, so its declared duty cycle is necessarily 100 percent.
fn tunnel_transmission_plan(
    jurisdiction: Jurisdiction,
    center_frequency_hz: f64,
    eirp_dbm: f64,
    sample_rate_hz: f64,
    deviation_hz: f32,
    samples_per_symbol: usize,
) -> std::result::Result<(TransmissionPlan, RadioParams), SdrError> {
    if !sample_rate_hz.is_finite() || sample_rate_hz <= 0.0 {
        return Err(SdrError::Config(format!(
            "tunnel sample rate must be finite and greater than 0 Hz (got {sample_rate_hz:?})"
        )));
    }
    let sample_rate = sample_rate_hz as f32;
    if !sample_rate.is_finite() || sample_rate <= 0.0 {
        return Err(SdrError::Config(format!(
            "tunnel sample rate cannot be represented as a positive finite f32 (got {sample_rate_hz:?} Hz)"
        )));
    }
    if !deviation_hz.is_finite() || deviation_hz < 0.0 {
        return Err(SdrError::Config(format!(
            "tunnel deviation must be finite and non-negative (got {deviation_hz:?} Hz)"
        )));
    }
    if samples_per_symbol == 0 {
        return Err(SdrError::Config(
            "tunnel samples per symbol must be greater than 0".to_string(),
        ));
    }

    let params = RadioParams {
        sample_rate,
        deviation_hz,
        samples_per_symbol,
    };
    let occupied_bandwidth_hz = 2.0 * f64::from(params.deviation_hz)
        + f64::from(params.sample_rate) / params.samples_per_symbol as f64;
    Ok((
        TransmissionPlan {
            jurisdiction,
            center_frequency_hz,
            occupied_bandwidth_hz,
            eirp_dbm,
            duty_cycle_pct: 100.0,
            encrypted: true,
        },
        params,
    ))
}

#[derive(Debug, Clone, Copy)]
struct TunnelEndpoint {
    local_addr: u8,
    peer: u8,
    mtu: usize,
    stdio: bool,
    listen: Option<u16>,
}

/// Build the bridge over `sdr` and pump the selected endpoint through it.
fn run_tunnel<D: SdrDriver>(
    sdr: D,
    endpoint: TunnelEndpoint,
    params: RadioParams,
    plan: TransmissionPlan,
) -> Result<(), Box<dyn std::error::Error>> {
    // `peer` is used as given. Pass 255 (broadcast) when the far end is this
    // same station — e.g. a radio in internal loopback — so the frame is not
    // filtered out by destination address when it returns.
    let link = RadioLink::new(sdr, endpoint.local_addr, endpoint.peer, params);
    let checked_link = PolicyCheckedInterface::new(link, plan);
    let mut bridge = StreamBridge::with_mtu(checked_link, endpoint.mtu);

    if endpoint.stdio {
        eprintln!("Piping stdin/stdout over the link (Ctrl-C to stop)...");
        run_stdio_tunnel(&mut bridge)
    } else if let Some(port) = endpoint.listen {
        eprintln!("Waiting for a TCP connection on port {port}...");
        run_tcp_tunnel(&mut bridge, port)
    } else {
        Ok(())
    }
}

/// Pipe stdin/stdout over the link — the OpenSSH `ProxyCommand` contract.
///
/// stdin is read on its own thread so a blocking read cannot stall the radio
/// side; the main loop polls both directions with a bounded sleep.
fn run_stdio_tunnel<I: sdr_mesh::node::MeshInterface>(
    bridge: &mut StreamBridge<I>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{Read, Write};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 4096];
        loop {
            match stdin.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut stdout = std::io::stdout();
    let mut stdin_open = true;
    loop {
        // Local -> radio
        loop {
            match rx.try_recv() {
                Ok(chunk) => bridge.write(&chunk)?,
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    stdin_open = false;
                    break;
                }
            }
        }

        // Radio -> local
        let received = bridge.read(TUNNEL_READ_MAX)?;
        if !received.is_empty() {
            stdout.write_all(&received)?;
            stdout.flush()?;
        }

        if !stdin_open && received.is_empty() {
            break;
        }
        std::thread::sleep(TUNNEL_POLL);
    }
    Ok(())
}

/// Accept one TCP connection and pipe it over the link.
fn run_tcp_tunnel<I: sdr_mesh::node::MeshInterface>(
    bridge: &mut StreamBridge<I>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind(("0.0.0.0", port))?;
    // Announce readiness on stderr (stdout carries the tunnelled stream). This
    // is the signal a caller waits on before connecting — a fixed sleep would
    // be a guess about bind latency.
    eprintln!("Listening on TCP port {port}; waiting for a connection...");
    let (mut sock, peer) = listener.accept()?;
    eprintln!("Connection from {peer}; piping over the link...");
    sock.set_nonblocking(true)?;

    let mut buf = [0u8; 4096];
    loop {
        // Local -> radio
        match sock.read(&mut buf) {
            Ok(0) => break, // peer closed
            Ok(n) => bridge.write(&buf[..n])?,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e.into()),
        }

        // Radio -> local
        let received = bridge.read(TUNNEL_READ_MAX)?;
        if !received.is_empty() {
            sock.write_all(&received)?;
            sock.flush()?;
        }
        std::thread::sleep(TUNNEL_POLL);
    }
    Ok(())
}

/// Bind a Hamlib rigctl TCP server on `port` and serve connections until the
/// process is stopped.
async fn run_rigctl_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    println!(
        "Listening for Hamlib connections on TCP port {} (Ctrl-C to stop)...",
        listener.local_addr().map(|a| a.port()).unwrap_or(port)
    );
    loop {
        let (socket, _peer) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_rigctl_client(socket).await {
                log::debug!("rigctl client disconnected: {e}");
            }
        });
    }
}

/// Serve a single rigctl client: parse each command line with [`RigctlHandler`],
/// reply, and keep the connection open until the client sends `q`.
async fn handle_rigctl_client(socket: tokio::net::TcpStream) -> std::io::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (read_half, mut write_half) = socket.into_split();
    let mut lines = BufReader::new(read_half).lines();
    let mut handler = RigctlHandler::new(RigState::default());

    while let Some(line) = lines.next_line().await? {
        let response = match handler.handle_command(&format!("{line}\n")) {
            Ok(resp) => resp,
            Err(_) => "RPRT -1\n".to_string(),
        };
        if !response.is_empty() {
            write_half.write_all(response.as_bytes()).await?;
        }
        if line.trim() == "q" {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor, Seek, SeekFrom, Write};

    struct FaultWriter {
        inner: Cursor<Vec<u8>>,
        write_limit: Option<u64>,
        fail_seek: bool,
    }

    impl FaultWriter {
        fn fail_after_header() -> Self {
            Self {
                inner: Cursor::new(Vec::new()),
                write_limit: Some(44),
                fail_seek: false,
            }
        }

        fn fail_during_finalize() -> Self {
            Self {
                inner: Cursor::new(Vec::new()),
                write_limit: None,
                fail_seek: true,
            }
        }
    }

    impl Write for FaultWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            if let Some(limit) = self.write_limit {
                let remaining = limit.saturating_sub(self.inner.position()) as usize;
                if remaining == 0 {
                    return Err(io::Error::other("scripted sample write failure"));
                }
                return self.inner.write(&buffer[..buffer.len().min(remaining)]);
            }
            self.inner.write(buffer)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }

    impl Seek for FaultWriter {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            if self.fail_seek {
                return Err(io::Error::other("scripted finalization seek failure"));
            }
            self.inner.seek(position)
        }
    }

    #[test]
    fn test_write_audio_wav_propagates_write_and_finalize_errors() {
        let write_error =
            write_audio_wav(FaultWriter::fail_after_header(), 48_000, &[0.25]).unwrap_err();
        assert!(
            write_error.to_string().contains("sample write failure"),
            "unexpected write error: {write_error}"
        );

        let finalize_error =
            write_audio_wav(FaultWriter::fail_during_finalize(), 48_000, &[0.25]).unwrap_err();
        assert!(
            finalize_error
                .to_string()
                .contains("finalization seek failure"),
            "unexpected finalization error: {finalize_error}"
        );
    }

    #[test]
    fn test_wav_sample_rate_rejects_unrepresentable_boundaries() {
        for invalid in [f32::NAN, f32::INFINITY, -1.0, 0.0, 0.1, 4_294_967_296.0] {
            assert!(wav_sample_rate(invalid).is_err(), "accepted {invalid:?}");
        }
        assert_eq!(wav_sample_rate(0.5).unwrap(), 1);
        assert_eq!(wav_sample_rate(48_000.0).unwrap(), 48_000);
        assert_eq!(wav_sample_rate(4_294_967_040.0).unwrap(), 4_294_967_040);
    }

    #[test]
    fn test_tunnel_plan_derives_carson_bandwidth_and_full_duty() {
        let defaults = RadioParams::default();
        let (default, default_params) = tunnel_transmission_plan(
            Jurisdiction::US,
            915_000_000.0,
            -10.0,
            1_000_000.0,
            defaults.deviation_hz,
            defaults.samples_per_symbol,
        )
        .unwrap();
        assert_eq!(default.occupied_bandwidth_hz, 300_000.0);
        assert_eq!(default.duty_cycle_pct, 100.0);
        assert_eq!(default.eirp_dbm, -10.0);
        assert!(default.encrypted);
        assert_eq!(default_params.sample_rate, 1_000_000.0);

        let (non_default, _) =
            tunnel_transmission_plan(Jurisdiction::EU, 433_920_000.0, 0.0, 96_000.0, 4_800.0, 8)
                .unwrap();
        assert_eq!(non_default.occupied_bandwidth_hz, 21_600.0);
        assert_eq!(non_default.duty_cycle_pct, 100.0);

        let (boundary, boundary_params) = tunnel_transmission_plan(
            Jurisdiction::US,
            915_000_000.0,
            20.0,
            3_000_000.1,
            defaults.deviation_hz,
            defaults.samples_per_symbol,
        )
        .unwrap();
        assert_eq!(f64::from(boundary_params.sample_rate), 3_000_000.0);
        assert_eq!(boundary.occupied_bandwidth_hz, 500_000.0);
    }

    #[test]
    fn test_tunnel_plan_rejects_unusable_sample_rates() {
        for sample_rate_hz in [
            f64::NAN,
            f64::INFINITY,
            0.0,
            -1.0,
            f64::MAX,
            f64::MIN_POSITIVE,
        ] {
            let error = tunnel_transmission_plan(
                Jurisdiction::US,
                915_000_000.0,
                0.0,
                sample_rate_hz,
                RadioParams::default().deviation_hz,
                RadioParams::default().samples_per_symbol,
            )
            .unwrap_err();
            assert!(error.to_string().contains("sample rate"));
        }
    }
}
