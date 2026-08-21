//! Constant False Alarm Rate (CA-CFAR) adaptive signal detector.

/// Cell-Averaging Constant False Alarm Rate (CA-CFAR) detector.
#[derive(Debug, Clone)]
pub struct CaCfarDetector {
    guard_cells: usize,
    train_cells: usize,
    threshold_factor: f32, // Offset factor in dB
}

impl CaCfarDetector {
    /// Create a new CA-CFAR detector.
    /// - `guard_cells`: number of guard cells on each side of CUT (Cell Under Test)
    /// - `train_cells`: number of background noise training cells on each side
    /// - `threshold_factor_db`: SNR margin in dB above estimated background noise
    pub fn new(guard_cells: usize, train_cells: usize, threshold_factor_db: f32) -> Self {
        Self {
            guard_cells,
            train_cells,
            threshold_factor: threshold_factor_db,
        }
    }

    /// Detect active signal peaks in power spectrum in dB.
    /// Returns vector of (bin_index, power_db, noise_threshold_db).
    pub fn detect(&self, spectrum: &[f32]) -> Vec<(usize, f32, f32)> {
        let n = spectrum.len();
        let total_window = self.guard_cells + self.train_cells;
        if n <= 2 * total_window {
            return Vec::new();
        }

        let mut detections = Vec::new();

        for cut in total_window..(n - total_window) {
            let pwr_cut = spectrum[cut];

            // Leading training cells: [cut - total_window .. cut - guard_cells]
            let lead_start = cut - total_window;
            let lead_end = cut - self.guard_cells;
            let mut sum_noise = 0.0f32;
            for i in lead_start..lead_end {
                sum_noise += spectrum[i];
            }

            // Trailing training cells: [cut + guard_cells + 1 .. cut + total_window + 1]
            let trail_start = cut + self.guard_cells + 1;
            let trail_end = cut + total_window + 1;
            for i in trail_start..trail_end {
                sum_noise += spectrum[i];
            }

            let num_train = (self.train_cells * 2) as f32;
            let avg_noise_db = sum_noise / num_train;
            let threshold = avg_noise_db + self.threshold_factor;

            if pwr_cut > threshold {
                detections.push((cut, pwr_cut, threshold));
            }
        }

        detections
    }
}
