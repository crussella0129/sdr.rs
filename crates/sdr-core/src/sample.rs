//! Core sample types and conversions for SDR signal processing.

use num_complex::Complex;

/// 32-bit floating point complex sample (standard baseband representation).
pub type Complex32 = Complex<f32>;

/// 64-bit floating point complex sample (high-precision DSP representation).
pub type Complex64 = Complex<f64>;

/// 16-bit signed integer complex sample (standard hardware ADC/DAC representation, e.g. PlutoSDR).
pub type ComplexI16 = Complex<i16>;

/// 8-bit signed integer complex sample (e.g. HackRF format).
pub type ComplexI8 = Complex<i8>;

/// 8-bit unsigned integer complex sample (e.g. RTL-SDR format).
pub type ComplexU8 = Complex<u8>;

/// Trait implemented by types that can represent SDR IQ samples.
pub trait Sample: Copy + Clone + Send + Sync + 'static {
    /// Convert sample to normalized 32-bit complex float (`[-1.0, 1.0]`).
    fn to_complex32(self) -> Complex32;

    /// Create sample from normalized 32-bit complex float.
    fn from_complex32(val: Complex32) -> Self;
}

impl Sample for Complex32 {
    #[inline(always)]
    fn to_complex32(self) -> Complex32 {
        self
    }

    #[inline(always)]
    fn from_complex32(val: Complex32) -> Self {
        val
    }
}

impl Sample for Complex64 {
    #[inline(always)]
    fn to_complex32(self) -> Complex32 {
        Complex32::new(self.re as f32, self.im as f32)
    }

    #[inline(always)]
    fn from_complex32(val: Complex32) -> Self {
        Complex64::new(val.re as f64, val.im as f64)
    }
}

impl Sample for ComplexI16 {
    #[inline(always)]
    fn to_complex32(self) -> Complex32 {
        const SCALE: f32 = 1.0 / 32768.0;
        Complex32::new(self.re as f32 * SCALE, self.im as f32 * SCALE)
    }

    #[inline(always)]
    fn from_complex32(val: Complex32) -> Self {
        let re = (val.re * 32767.0).clamp(-32768.0, 32767.0) as i16;
        let im = (val.im * 32767.0).clamp(-32768.0, 32767.0) as i16;
        ComplexI16::new(re, im)
    }
}

impl Sample for ComplexI8 {
    #[inline(always)]
    fn to_complex32(self) -> Complex32 {
        const SCALE: f32 = 1.0 / 128.0;
        Complex32::new(self.re as f32 * SCALE, self.im as f32 * SCALE)
    }

    #[inline(always)]
    fn from_complex32(val: Complex32) -> Self {
        let re = (val.re * 127.0).clamp(-128.0, 127.0) as i8;
        let im = (val.im * 127.0).clamp(-128.0, 127.0) as i8;
        ComplexI8::new(re, im)
    }
}

impl Sample for ComplexU8 {
    #[inline(always)]
    fn to_complex32(self) -> Complex32 {
        const SCALE: f32 = 1.0 / 127.5;
        Complex32::new((self.re as f32 - 127.5) * SCALE, (self.im as f32 - 127.5) * SCALE)
    }

    #[inline(always)]
    fn from_complex32(val: Complex32) -> Self {
        let re = ((val.re + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
        let im = ((val.im + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
        ComplexU8::new(re, im)
    }
}

/// Convert a slice of arbitrary sample type to normalized `Complex32`.
pub fn convert_samples<S: Sample>(src: &[S], dst: &mut [Complex32]) {
    assert_eq!(src.len(), dst.len());
    for (s, d) in src.iter().zip(dst.iter_mut()) {
        *d = s.to_complex32();
    }
}

/// Convert raw interleaved bytes to Complex32 based on sample format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SampleFormat {
    /// 32-bit floating point complex (cf32_le / cf32)
    ComplexF32,
    /// 64-bit floating point complex (cf64_le / cf64)
    ComplexF64,
    /// 16-bit signed integer complex (cs16_le / cs16)
    ComplexI16,
    /// 8-bit signed integer complex (cs8)
    ComplexI8,
    /// 8-bit unsigned integer complex (cu8)
    ComplexU8,
    /// 32-bit floating point real (f32_le)
    RealF32,
    /// 16-bit signed integer real (s16_le)
    RealI16,
}
