//! Real-binary coverage for truthful demodulation output.

use sdr_core::sample::Complex32;
use sdr_hardware::wav::write_iq_wav;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let serial = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("sdr_cli_demod_{}_{}", std::process::id(), serial));
        std::fs::create_dir_all(&path).expect("create demod test directory");
        Self(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn write_fixture(path: &Path) {
    const SAMPLE_RATE: u32 = 48_000;
    let samples: Vec<_> = (0..4_096)
        .map(|index| {
            let phase = std::f32::consts::TAU * 1_000.0 * index as f32 / SAMPLE_RATE as f32;
            let amplitude = 0.5 + 0.25 * (phase * 0.1).sin();
            Complex32::new(amplitude * phase.cos(), amplitude * phase.sin())
        })
        .collect();
    write_iq_wav(path, SAMPLE_RATE, &samples).expect("write deterministic IQ fixture");
}

fn demod(input: &Path, mode: &str, output: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sdr-cli"));
    command
        .arg("demod")
        .arg("--input")
        .arg(input)
        .arg("--mode")
        .arg(mode);
    if let Some(path) = output {
        command.arg("--output").arg(path);
    }
    command.output().expect("run sdr-cli demod")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn test_cli_demod_audio_writes_real_mono_wav() {
    let dir = TestDir::new();
    let input = dir.join("capture.wav");
    let output_path = dir.join("audio.wav");
    write_fixture(&input);

    let output = demod(&input, "am", Some(&output_path));
    assert!(output.status.success(), "demod failed: {}", stderr(&output));
    assert!(stdout(&output).contains("Exported"));

    let mut reader = hound::WavReader::open(&output_path).expect("open exported WAV");
    let spec = reader.spec();
    assert_eq!(spec.channels, 1);
    assert_eq!(spec.sample_rate, 48_000);
    let samples = reader
        .samples::<i16>()
        .collect::<hound::Result<Vec<_>>>()
        .expect("decode every exported PCM sample");
    assert_eq!(samples.len(), 4_096);
    assert!(stdout(&output).contains("Exported 4096 audio samples"));
}

#[test]
fn test_cli_demod_fsk_output_is_rejected_without_file() {
    let dir = TestDir::new();
    let input = dir.join("capture.wav");
    let output_path = dir.join("bits.bin");
    write_fixture(&input);

    let output = demod(&input, "fsk", Some(&output_path));
    assert!(!output.status.success());
    assert!(!output_path.exists());
    assert!(!stdout(&output).contains("Exported"));
    assert!(stderr(&output).to_lowercase().contains("unsupported"));
}

#[test]
fn test_cli_demod_unknown_mode_fails_without_output() {
    let dir = TestDir::new();
    let input = dir.join("capture.wav");
    let output_path = dir.join("unknown.wav");
    write_fixture(&input);

    let output = demod(&input, "definitely-not-a-mode", Some(&output_path));
    assert!(!output.status.success());
    assert!(!output_path.exists());
    assert!(!stdout(&output).contains("Exported"));
    assert!(stderr(&output).contains("Unknown demodulation mode"));
}

#[test]
fn test_cli_demod_incompatible_extension_fails_without_output() {
    let dir = TestDir::new();
    let input = dir.join("capture.wav");
    let output_path = dir.join("audio.txt");
    write_fixture(&input);

    let output = demod(&input, "am", Some(&output_path));
    assert!(!output.status.success());
    assert!(!output_path.exists());
    assert!(!stdout(&output).contains("Exported"));
    assert!(stderr(&output).to_lowercase().contains("wav"));
}

#[test]
fn test_cli_demod_wav_open_failure_returns_nonzero_without_success_claim() {
    let dir = TestDir::new();
    let input = dir.join("capture.wav");
    let output_path = dir.join("blocked.wav");
    write_fixture(&input);
    std::fs::create_dir(&output_path).expect("create directory at output path");

    let output = demod(&input, "am", Some(&output_path));
    assert!(!output.status.success());
    assert!(!stdout(&output).contains("Exported"));
    assert!(
        !stderr(&output).trim().is_empty(),
        "the underlying file-open error must be surfaced"
    );
}

#[test]
fn test_cli_demod_without_output_reports_typed_count() {
    let dir = TestDir::new();
    let input = dir.join("capture.wav");
    write_fixture(&input);

    let audio = demod(&input, "am", None);
    assert!(audio.status.success(), "audio: {}", stderr(&audio));
    assert!(stdout(&audio).contains("audio samples produced"));
    assert!(!stdout(&audio).contains("Exported"));

    let bits = demod(&input, "fsk", None);
    assert!(bits.status.success(), "bits: {}", stderr(&bits));
    assert!(stdout(&bits).contains("bits"));
    assert!(!stdout(&bits).contains("Exported"));
}
