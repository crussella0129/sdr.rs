//! Frequency Shift Keying (FSK/GFSK) digital demodulators.
//!
//! Two are provided:
//!
//! - [`FskDemod`] averages the instantaneous frequency over a **fixed** number
//!   of samples per symbol. Simple and exact when the stream begins on a symbol
//!   boundary, but it has no timing recovery, so a mid-symbol start or a clock
//!   offset corrupts the output.
//! - [`FskTimingDemod`] runs a Gardner timing-recovery loop and tracks the
//!   symbol clock, so it tolerates an arbitrary start phase and a transmitter/
//!   receiver clock offset. Prefer it whenever the two ends do not share a clock.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};
use sdr_dsp::clock_recovery::GardnerClockRecovery;
use std::f32::consts::PI;

/// 2-FSK Digital Demodulator and Bit Slicer.
#[derive(Debug, Clone)]
pub struct FskDemod {
    sample_rate: f32,
    _deviation: f32,
    samples_per_symbol: usize,
    last_sample: Complex32,
    sample_counter: usize,
    accumulated_freq: f32,
}

impl FskDemod {
    /// Create a new 2-FSK demodulator.
    pub fn new(sample_rate: f32, deviation: f32, samples_per_symbol: usize) -> Self {
        Self {
            sample_rate,
            _deviation: deviation,
            samples_per_symbol: samples_per_symbol.max(1),
            last_sample: Complex32::new(1.0, 0.0),
            sample_counter: 0,
            accumulated_freq: 0.0,
        }
    }

    /// Process a stream of IQ samples and return demodulated binary bits (0 or 1).
    pub fn demod_bits(&mut self, input: &[Complex32], output_bits: &mut Vec<u8>) {
        for &s in input {
            let prod = s * self.last_sample.conj();
            let phase_diff = prod.im.atan2(prod.re);
            self.last_sample = s;

            let instantaneous_freq = (phase_diff * self.sample_rate) / (2.0 * PI);
            self.accumulated_freq += instantaneous_freq;
            self.sample_counter += 1;

            if self.sample_counter >= self.samples_per_symbol {
                let mean_freq = self.accumulated_freq / self.sample_counter as f32;
                // Slicer: positive frequency offset -> bit 1, negative -> bit 0
                let bit = if mean_freq >= 0.0 { 1u8 } else { 0u8 };
                output_bits.push(bit);

                self.sample_counter = 0;
                self.accumulated_freq = 0.0;
            }
        }
    }

    pub fn reset(&mut self) {
        self.last_sample = Complex32::new(1.0, 0.0);
        self.sample_counter = 0;
        self.accumulated_freq = 0.0;
    }
}

impl Block<Complex32, u8> for FskDemod {
    fn process(&mut self, input: &[Complex32], output: &mut Vec<u8>) -> Result<(usize, usize)> {
        output.clear();
        output.reserve(input.len() / self.samples_per_symbol + 2);
        self.demod_bits(input, output);
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}

/// 2-FSK demodulator with symbol-timing recovery.
///
/// The frequency discriminator runs **first**, turning constant-envelope FSK
/// into a real-valued PAM signal; only then does the Gardner loop have symbol
/// transitions to lock onto. (Feeding raw FSK IQ to a Gardner detector does not
/// work: every sample has the same magnitude and only the rotation rate carries
/// information.)
///
/// Because the loop tracks the clock rather than counting samples, this
/// tolerates an arbitrary start phase and a transmitter/receiver clock offset.
///
/// **Drift tolerance depends on burst length**, because the loop needs time to
/// converge and residual error accumulates: a short burst (~80 bits) is
/// bit-exact to ±0.5%, while a full frame (~256 bits, where every bit must
/// survive for CRC-32 to pass) holds to about **±0.1%**. Tolerance is also
/// slightly asymmetric — a receiver clock running *fast* is the tighter
/// direction. Loop-gain tuning does not widen this, so the limit is structural
/// to this Gardner implementation rather than a matter of configuration.
///
/// Even ±0.1% (±1000 ppm) is one to two orders of magnitude beyond the
/// ±10–50 ppm of a real crystal oscillator.
///
/// The recovered stream may lead or lag the transmitted one by a symbol while
/// the loop settles; a frame-level sync-word search absorbs that.
#[derive(Debug, Clone)]
pub struct FskTimingDemod {
    gardner: GardnerClockRecovery,
    last_sample: Complex32,
    scratch: Vec<Complex32>,
}

impl FskTimingDemod {
    /// Create a demodulator for `samples_per_symbol`, with Gardner loop gains.
    ///
    /// `gain_mu` (timing) and `gain_omega` (rate) of `0.01` / `0.001` are the
    /// defaults used elsewhere in the project and are what the ±0.5% figure was
    /// measured with.
    pub fn new(samples_per_symbol: f32, gain_mu: f32, gain_omega: f32) -> Self {
        Self {
            gardner: GardnerClockRecovery::new(samples_per_symbol, gain_mu, gain_omega),
            last_sample: Complex32::new(1.0, 0.0),
            scratch: Vec::new(),
        }
    }

    /// Create a demodulator with the project's default loop gains.
    pub fn with_defaults(samples_per_symbol: f32) -> Self {
        Self::new(samples_per_symbol, 0.01, 0.001)
    }

    /// Demodulate IQ into bits, recovering symbol timing. State carries across
    /// calls, so a stream may be fed in arbitrary chunks.
    pub fn demod_bits(&mut self, input: &[Complex32], output_bits: &mut Vec<u8>) {
        // IQ -> instantaneous frequency, presented as a real-valued PAM signal.
        self.scratch.clear();
        self.scratch.reserve(input.len());
        for &s in input {
            let prod = s * self.last_sample.conj();
            self.last_sample = s;
            self.scratch
                .push(Complex32::new(prod.im.atan2(prod.re), 0.0));
        }

        let mut symbols = Vec::with_capacity(input.len() / 2 + 8);
        self.gardner.process_samples(&self.scratch, &mut symbols);

        // Positive frequency deviation -> bit 1, negative -> bit 0.
        output_bits.extend(symbols.iter().map(|s| u8::from(s.re >= 0.0)));
    }

    pub fn reset(&mut self) {
        self.gardner.reset();
        self.last_sample = Complex32::new(1.0, 0.0);
        self.scratch.clear();
    }
}

impl Block<Complex32, u8> for FskTimingDemod {
    fn process(&mut self, input: &[Complex32], output: &mut Vec<u8>) -> Result<(usize, usize)> {
        output.clear();
        self.demod_bits(input, output);
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}
