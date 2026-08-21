//! Mock SDR hardware driver for deterministic, offline testing and TX loopback.

use crate::driver::{GainMode, SdrDriver};
use sdr_core::sample::Complex32;
use sdr_core::traits::Result;
use std::collections::VecDeque;
use std::f32::consts::TAU;

/// Signal generation pattern for Mock SDR.
#[derive(Debug, Clone)]
pub enum MockSignal {
    /// Pure sinusoidal tone at given baseband offset in Hz
    Tone { offset_hz: f64, amplitude: f32 },
    /// Multi-tone comb
    MultiTone { tones: Vec<(f64, f32)> },
    /// Constant DC level
    Constant(Complex32),
    /// Zeroes / Silent stream
    Silence,
    /// Loopback buffer mode (samples written to TX are read back on RX)
    Loopback,
}

/// A simulated SDR hardware device generating synthetic IQ samples or looping back TX to RX.
#[derive(Debug, Clone)]
pub struct MockSdr {
    frequency: f64,
    sample_rate: f64,
    bandwidth: f64,
    gain: f64,
    gain_mode: GainMode,
    active: bool,
    tx_active: bool,
    signal: MockSignal,
    phase: f32,
    tx_queue: VecDeque<Complex32>,
}

impl MockSdr {
    pub fn new(sample_rate: f64, frequency: f64) -> Self {
        Self {
            frequency,
            sample_rate,
            bandwidth: sample_rate * 0.8,
            gain: 30.0,
            gain_mode: GainMode::Manual,
            active: false,
            tx_active: false,
            signal: MockSignal::Tone {
                offset_hz: 10000.0,
                amplitude: 0.8,
            },
            phase: 0.0,
            tx_queue: VecDeque::new(),
        }
    }

    /// Set signal generation pattern.
    pub fn set_signal(&mut self, signal: MockSignal) {
        self.signal = signal;
        self.phase = 0.0;
    }

    /// Enable TX to RX loopback mode.
    pub fn enable_loopback(&mut self) {
        self.signal = MockSignal::Loopback;
    }
}

impl SdrDriver for MockSdr {
    fn name(&self) -> &str {
        "Mock SDR Driver"
    }

    fn set_frequency(&mut self, _channel: usize, freq_hz: f64) -> Result<()> {
        self.frequency = freq_hz;
        Ok(())
    }

    fn set_sample_rate(&mut self, _channel: usize, rate_hz: f64) -> Result<()> {
        self.sample_rate = rate_hz;
        Ok(())
    }

    fn set_bandwidth(&mut self, _channel: usize, bw_hz: f64) -> Result<()> {
        self.bandwidth = bw_hz;
        Ok(())
    }

    fn set_gain(&mut self, _channel: usize, gain_db: f64) -> Result<()> {
        self.gain = gain_db;
        Ok(())
    }

    fn set_gain_mode(&mut self, _channel: usize, mode: GainMode) -> Result<()> {
        self.gain_mode = mode;
        Ok(())
    }

    fn start_rx(&mut self) -> Result<()> {
        self.active = true;
        Ok(())
    }

    fn stop_rx(&mut self) -> Result<()> {
        self.active = false;
        Ok(())
    }

    fn start_tx(&mut self) -> Result<()> {
        self.tx_active = true;
        Ok(())
    }

    fn stop_tx(&mut self) -> Result<()> {
        self.tx_active = false;
        Ok(())
    }

    fn has_tx(&self) -> bool {
        true
    }

    fn write_samples(&mut self, buffer: &[Complex32]) -> Result<usize> {
        if !self.tx_active {
            return Ok(0);
        }
        self.tx_queue.extend(buffer.iter().copied());
        Ok(buffer.len())
    }

    fn read_samples(&mut self, buffer: &mut [Complex32]) -> Result<usize> {
        if !self.active || self.sample_rate <= 0.0 {
            return Ok(0);
        }

        let n = buffer.len();
        match &self.signal {
            MockSignal::Loopback => {
                let mut count = 0;
                for s in buffer.iter_mut() {
                    if let Some(sample) = self.tx_queue.pop_front() {
                        *s = sample;
                        count += 1;
                    } else {
                        *s = Complex32::default();
                    }
                }
                Ok(count)
            }
            MockSignal::Tone {
                offset_hz,
                amplitude,
            } => {
                let phase_inc = (*offset_hz as f32 / self.sample_rate as f32) * TAU;
                for s in buffer.iter_mut() {
                    let (sin_val, cos_val) = self.phase.sin_cos();
                    *s = Complex32::new(cos_val * *amplitude, sin_val * *amplitude);
                    self.phase = (self.phase + phase_inc).rem_euclid(TAU);
                }
                Ok(n)
            }
            MockSignal::MultiTone { tones } => {
                for s in buffer.iter_mut() {
                    let mut acc = Complex32::default();
                    for &(offset, amp) in tones {
                        let cur_phase = (self.phase * (offset as f32 / 1000.0)).rem_euclid(TAU);
                        let (sin_val, cos_val) = cur_phase.sin_cos();
                        acc += Complex32::new(cos_val * amp, sin_val * amp);
                    }
                    *s = acc;
                    self.phase = (self.phase + 0.1).rem_euclid(TAU);
                }
                Ok(n)
            }
            MockSignal::Constant(c) => {
                buffer.fill(*c);
                Ok(n)
            }
            MockSignal::Silence => {
                buffer.fill(Complex32::default());
                Ok(n)
            }
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn teardown(&mut self) -> Result<()> {
        self.active = false;
        self.tx_active = false;
        self.tx_queue.clear();
        Ok(())
    }
}
