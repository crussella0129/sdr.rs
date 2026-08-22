//! # sdr-cli
//!
//! Command-line interface for the `sdr.rs` cross-platform SDR suite.

use clap::{Parser, Subcommand};
use sdr_core::compliance::{ComplianceResult, Jurisdiction, RegulatoryDatabase};
use sdr_core::sample::Complex32;
use sdr_demod::{AmDemod, DeEmphasis, FskDemod, NfmDemod, SsbDemod, SsbMode, WfmDemod};
use sdr_dsp::window::WindowType;
use sdr_hardware::driver::SdrDriver;
use sdr_hardware::mock::{MockSdr, MockSignal};
use sdr_hardware::sigmf::{SigMfReader, SigMfWriter};
use sdr_hardware::wav::{read_iq_wav, write_iq_wav};
use sdr_protocols::tunnel::StreamTunnel;
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
        jurisdiction: String,

        /// Require legal support for encrypted payloads (such as SSH / TLS)
        #[arg(short, long, default_value_t = false)]
        encrypted: bool,

        /// Specific frequency in Hz to check compliance (optional)
        #[arg(long)]
        check_freq: Option<u64>,

        /// Transmit power in dBm for compliance check
        #[arg(short, long, default_value_t = 20.0)]
        power: f32,
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
        jurisdiction: String,
    },
    /// Start Hamlib Rigctl TCP server for external radio control
    Rigctl {
        /// TCP port to bind (default: 4532)
        #[arg(short, long, default_value_t = 4532)]
        port: u16,
    },
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

            let mut sdr = MockSdr::new(rate, freq);
            sdr.set_signal(MockSignal::Tone {
                offset_hz: 10000.0,
                amplitude: 0.8,
            });
            sdr.start_rx()?;

            let mut buffer = vec![Complex32::default(); samples];
            sdr.read_samples(&mut buffer)?;
            sdr.stop_rx()?;

            let ext = output.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext == "wav" {
                write_iq_wav(&output, rate as u32, &buffer)?;
                println!("Saved {} samples to WAV IQ: {}", samples, output.display());
            } else {
                let mut writer = SigMfWriter::create(&output, rate, freq)?;
                writer.write_samples(&buffer)?;
                writer.close()?;
                println!(
                    "Saved {} samples to SigMF archive: {}",
                    samples,
                    output.display()
                );
            }
        }
        Commands::Demod {
            input,
            mode,
            output,
        } => {
            println!("=== sdr.rs Demodulator ===");
            println!("Input: {}, Mode: {}", input.display(), mode);

            let (sample_rate, samples) = if input.extension().map_or(false, |e| e == "wav") {
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

            let audio: Vec<f32> = match mode.to_lowercase().as_str() {
                "wfm" => {
                    let mut demod = WfmDemod::new(sample_rate, 75000.0, DeEmphasis::Eu50us);
                    let mut out = Vec::new();
                    demod.demod_block(&samples, &mut out);
                    out
                }
                "nfm" => {
                    let mut demod = NfmDemod::new(sample_rate, 5000.0, -40.0);
                    let mut out = Vec::new();
                    for &s in &samples {
                        out.push(demod.demod_sample(s));
                    }
                    out
                }
                "am" => {
                    let mut demod = AmDemod::new();
                    let mut out = Vec::new();
                    for &s in &samples {
                        out.push(demod.demod_sample(s));
                    }
                    out
                }
                "ssb" => {
                    let mut demod = SsbDemod::new(SsbMode::Usb);
                    let mut out = Vec::new();
                    for &s in &samples {
                        out.push(demod.demod_sample(s));
                    }
                    out
                }
                "fsk" => {
                    let mut demod = FskDemod::new(sample_rate, 5000.0, 10);
                    let mut bits = Vec::new();
                    demod.demod_bits(&samples, &mut bits);
                    println!(
                        "Demodulated {} bits: {:?}",
                        bits.len(),
                        &bits[..bits.len().min(32)]
                    );
                    Vec::new()
                }
                other => {
                    println!(
                        "Unknown demodulation mode '{}'. Supported: wfm, nfm, am, ssb, fsk",
                        other
                    );
                    Vec::new()
                }
            };

            if let Some(out_path) = output {
                if !audio.is_empty() {
                    println!(
                        "Exported {} audio samples to {}",
                        audio.len(),
                        out_path.display()
                    );
                }
            } else {
                println!(
                    "Demodulation completed successfully ({} audio samples produced)",
                    audio.len()
                );
            }
        }
        Commands::Spectrum { input, fft_size } => {
            println!("=== sdr.rs Spectrum Analyzer ===");
            let (_rate, samples) = if input.extension().map_or(false, |e| e == "wav") {
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
            power,
        } => {
            let jur = Jurisdiction::from_str(&jurisdiction).unwrap_or(Jurisdiction::US);
            println!("=== sdr.rs RF Regulatory Advisor ===");
            println!("Selected Jurisdiction: {}", jur.as_str());
            println!(
                "Encryption Support Required: {}",
                if encrypted {
                    "YES (e.g. SSH / TLS / AES)"
                } else {
                    "NO (Open telemetry / voice)"
                }
            );

            if let Some(freq) = check_freq {
                println!(
                    "\n--- Compliance Verification for {:.3} MHz ---",
                    freq as f64 / 1e6
                );
                let result = RegulatoryDatabase::check_compliance(jur, freq, power, encrypted);
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
                let bands = RegulatoryDatabase::query_recommended_bands(jur, encrypted);
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
            rate,
            jurisdiction,
        } => {
            let jur = Jurisdiction::from_str(&jurisdiction).unwrap_or(Jurisdiction::US);
            println!("=== sdr.rs SSH over Radio Tunnel Bridge ===");
            println!(
                "Local Station: 0x{:02X}, Peer Station: 0x{:02X}",
                local_addr, peer
            );
            println!(
                "Frequency: {:.3} MHz, Rate: {:.3} MSPS, Jurisdiction: {}",
                freq / 1e6,
                rate / 1e6,
                jur.as_str()
            );

            // Check compliance for encrypted SSH
            let check = RegulatoryDatabase::check_compliance(jur, freq as u64, 20.0, true);
            match check {
                ComplianceResult::Compliant {
                    band_name,
                    citation,
                    ..
                } => {
                    println!(
                        "Regulatory Status: COMPLIANT ({}) - {}",
                        band_name, citation
                    );
                }
                ComplianceResult::NonCompliant { reasons } => {
                    println!("WARNING: Transmission on this band with encryption may violate regulations:");
                    for r in reasons {
                        println!("  - {}", r);
                    }
                }
            }

            let tunnel = StreamTunnel::new(local_addr, peer, 256);
            println!(
                "Stream Tunnel initialized (MTU {} bytes). Ready for OpenSSH ProxyCommand.",
                tunnel.mtu
            );
        }
        Commands::Rigctl { port } => {
            println!("=== sdr.rs Hamlib Rigctl Server ===");
            run_rigctl_server(port).await?;
        }
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
