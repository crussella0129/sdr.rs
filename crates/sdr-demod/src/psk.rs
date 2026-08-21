//! Phase Shift Keying (BPSK / QPSK) Digital Demodulator.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};
use sdr_dsp::clock_recovery::GardnerClockRecovery;
use sdr_dsp::costas::{CostasLoop, ModulationOrder};

/// PSK Demodulator (BPSK and QPSK).
#[derive(Debug, Clone)]
pub struct PskDemod {
    costas: CostasLoop,
    clock_recovery: GardnerClockRecovery,
    order: ModulationOrder,
}

impl PskDemod {
    /// Create a new PSK demodulator (e.g. samples_per_symbol = 4.0).
    pub fn new(samples_per_symbol: f32, order: ModulationOrder) -> Self {
        Self {
            costas: CostasLoop::new(0.02, order),
            clock_recovery: GardnerClockRecovery::new(samples_per_symbol, 0.01, 0.001),
            order,
        }
    }

    /// Demodulate IQ stream to bit stream.
    pub fn demod_bits(&mut self, input: &[Complex32], output_bits: &mut Vec<u8>) {
        let mut synchronized = Vec::new();
        self.clock_recovery
            .process_samples(input, &mut synchronized);

        for &sym in &synchronized {
            let locked_sym = self.costas.process_sample(sym);
            match self.order {
                ModulationOrder::Bpsk => {
                    let bit = if locked_sym.re >= 0.0 { 1u8 } else { 0u8 };
                    output_bits.push(bit);
                }
                ModulationOrder::Qpsk => {
                    // Gray coded QPSK:
                    // Quadrant 0 (+,+): 00
                    // Quadrant 1 (-,+): 01
                    // Quadrant 2 (-,-): 11
                    // Quadrant 3 (+,-): 10
                    let b0 = if locked_sym.re >= 0.0 { 0u8 } else { 1u8 };
                    let b1 = if locked_sym.im >= 0.0 { 0u8 } else { 1u8 };
                    output_bits.push(b0);
                    output_bits.push(b1);
                }
                ModulationOrder::Psk8 => {
                    let angle = locked_sym.im.atan2(locked_sym.re);
                    let octant = ((angle / (std::f32::consts::PI / 4.0)).round() as i32)
                        .rem_euclid(8) as u8;
                    output_bits.push((octant >> 2) & 1);
                    output_bits.push((octant >> 1) & 1);
                    output_bits.push(octant & 1);
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.costas.reset();
        self.clock_recovery.reset();
    }
}

impl Block<Complex32, u8> for PskDemod {
    fn process(&mut self, input: &[Complex32], output: &mut Vec<u8>) -> Result<(usize, usize)> {
        output.clear();
        self.demod_bits(input, output);
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}
