//! Standard DSP window functions for FIR filter design and spectral analysis.

use std::f64::consts::PI;

/// Window function types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Rectangular,
    Hann,
    Hamming,
    Blackman,
    BlackmanHarris,
    Kaiser(u32), // beta parameter scaled (e.g. beta = 5.0 -> 500)
}

/// Generate window coefficients for length `n`.
pub fn generate_window(window_type: WindowType, n: usize) -> Vec<f32> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![1.0];
    }

    let mut w = Vec::with_capacity(n);
    let n_minus_1 = (n - 1) as f64;

    match window_type {
        WindowType::Rectangular => {
            w.resize(n, 1.0);
        }
        WindowType::Hann => {
            for i in 0..n {
                let val = 0.5 * (1.0 - (2.0 * PI * i as f64 / n_minus_1).cos());
                w.push(val as f32);
            }
        }
        WindowType::Hamming => {
            for i in 0..n {
                let val = 0.54 - 0.46 * (2.0 * PI * i as f64 / n_minus_1).cos();
                w.push(val as f32);
            }
        }
        WindowType::Blackman => {
            for i in 0..n {
                let a0 = 0.42;
                let a1 = 0.50;
                let a2 = 0.08;
                let val = a0 - a1 * (2.0 * PI * i as f64 / n_minus_1).cos()
                    + a2 * (4.0 * PI * i as f64 / n_minus_1).cos();
                w.push(val as f32);
            }
        }
        WindowType::BlackmanHarris => {
            // 4-term Blackman-Harris window (> 92 dB sidelobe suppression)
            const A0: f64 = 0.35875;
            const A1: f64 = 0.48829;
            const A2: f64 = 0.14128;
            const A3: f64 = 0.01168;
            for i in 0..n {
                let val = A0 - A1 * (2.0 * PI * i as f64 / n_minus_1).cos()
                    + A2 * (4.0 * PI * i as f64 / n_minus_1).cos()
                    - A3 * (6.0 * PI * i as f64 / n_minus_1).cos();
                w.push(val as f32);
            }
        }
        WindowType::Kaiser(scaled_beta) => {
            let beta = scaled_beta as f64 / 100.0;
            let i0_beta = bessel_i0(beta);
            for i in 0..n {
                let k = (2.0 * i as f64 / n_minus_1) - 1.0;
                let arg = beta * (1.0 - k * k).max(0.0).sqrt();
                let val = bessel_i0(arg) / i0_beta;
                w.push(val as f32);
            }
        }
    }
    w
}

/// Zero-th order modified Bessel function of the first kind $I_0(x)$.
pub fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let x_half = x / 2.0;
    for k in 1..50 {
        term *= (x_half / k as f64) * (x_half / k as f64);
        sum += term;
        if term < 1e-12 * sum {
            break;
        }
    }
    sum
}
