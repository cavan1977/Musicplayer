//! 64-band FFT-based spectrum analyzer for real-time audio visualization.
//! Uses rustfft for FFT computation; thread-safe for audio thread.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use rustfft::{Fft, FftPlanner, num_complex::Complex};

/// Number of frequency bands for visualization
pub const BAND_COUNT: usize = 64;

/// FFT size - must be power of two
const FFT_SIZE: usize = 1024;

/// Smoothing factor (0.0 = no smoothing, close to 1.0 = heavy smoothing)
const SMOOTHING: f32 = 0.85;

/// Hann window
fn hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / size as f32).cos())
        .collect()
}

/// Thread-safe output container that stores the latest smoothed band values.
/// This can be shared between the audio thread (writer) and the UI thread (reader).
pub struct SpectrumOutput {
    bands: [AtomicU32; BAND_COUNT],
}

impl SpectrumOutput {
    pub fn new() -> Self {
        // Initialize all bands to zero
        let bands: [AtomicU32; BAND_COUNT] = [(); BAND_COUNT].map(|_| AtomicU32::new(0));
        Self { bands }
    }

    /// Write a band value (called from the analyzer).
    pub fn set_band(&self, band: usize, value: f32) {
        if band < BAND_COUNT {
            self.bands[band].store(value.min(1.0).to_bits(), Ordering::Relaxed);
        }
    }

    /// Retrieve all band values as an array.
    pub fn get_bands(&self) -> [f32; BAND_COUNT] {
        let mut out = [0.0f32; BAND_COUNT];
        for (i, atomic) in self.bands.iter().enumerate() {
            out[i] = f32::from_bits(atomic.load(Ordering::Relaxed));
        }
        out
    }
}

pub struct SpectrumAnalyzer {
    /// FFT planner
    fft: Arc<dyn Fft<f32>>,
    /// FFT buffer (complex input/output)
    fft_buf: Vec<Complex<f32>>,
    /// Hann window coefficients
    window: Vec<f32>,
    /// Circular buffer of recent mono samples
    ring_buf: Vec<f32>,
    /// Write position in ring buffer
    ring_pos: usize,
    /// Shared output for band values
    output: Arc<SpectrumOutput>,
    /// Smoothing state per band
    smoothed: Vec<f32>,
    /// Mapping from FFT bin index (0..bin_count) to band index (0..BAND_COUNT)
    bin_to_band: Vec<u8>,
    /// Number of FFT bins that contribute to each band (for normalization)
    band_bin_count: Vec<f32>,
}

impl SpectrumAnalyzer {
    /// Create a new analyzer bound to the given sample rate and output.
    pub fn new(sample_rate: u32, output: Arc<SpectrumOutput>) -> Self {
        // FFT planner
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        // Allocate buffers
        let fft_buf = vec![Complex { re: 0.0, im: 0.0 }; FFT_SIZE];
        let window = hann_window(FFT_SIZE);
        let ring_buf = vec![0.0; FFT_SIZE];
        let ring_pos = 0;

        // Smoothing states per band
        let smoothed = vec![0.0; BAND_COUNT];

        // Compute bin -> band mapping and bin counts per band
        let (bin_to_band, band_bin_count) = Self::compute_band_mapping(sample_rate);

        Self {
            fft,
            fft_buf,
            window,
            ring_buf,
            ring_pos,
            output,
            smoothed,
            bin_to_band,
            band_bin_count,
        }
    }

    /// Compute band edges and bin mapping based on sample rate.
    fn compute_band_mapping(sample_rate: u32) -> (Vec<u8>, Vec<f32>) {
        let nyquist = sample_rate as f32 / 2.0;
        let low_cutoff = 20.0f32; // ignore sub-20 Hz as it's rarely content
        let bin_count = FFT_SIZE / 2 + 1; // bins from 0 to Nyquist inclusive
        let freq_per_bin = sample_rate as f32 / FFT_SIZE as f32;

        // Create band edges (log spaced)
        let mut edges = Vec::with_capacity(BAND_COUNT + 1);
        for i in 0..=BAND_COUNT {
            let p = i as f32 / BAND_COUNT as f32;
            let log_freq = low_cutoff.log10()
                + p * (nyquist.log10() - low_cutoff.log10());
            edges.push(10.0f32.powf(log_freq));
        }

        // Map each bin to a band
        let mut bin_to_band = Vec::with_capacity(bin_count);
        let mut band_bin_count = vec![0.0; BAND_COUNT];

        for bin in 0..bin_count {
            let freq = bin as f32 * freq_per_bin;
            // Find the band such that edges[band] <= freq < edges[band+1]
            let mut band = 0;
            while band < BAND_COUNT - 1 && freq >= edges[band + 1] {
                band += 1;
            }
            bin_to_band.push(band as u8);
            band_bin_count[band] += 1.0;
        }

        (bin_to_band, band_bin_count)
    }

    /// Analyze a block of interleaved audio samples.
    /// `samples` – PCM f32 samples (may be interleaved for multi‑channel)
    /// `channels` – number of channels (1 for mono, 2 for stereo, etc.)
    pub fn analyze(&mut self, samples: &[f32], channels: usize) {
        // Mix down to mono while feeding into ring buffer
        if channels == 1 {
            for &sample in samples {
                self.ring_buf[self.ring_pos] = sample;
                self.ring_pos = (self.ring_pos + 1) % FFT_SIZE;
            }
        } else {
            let mut i = 0;
            while i < samples.len() {
                // Use the first channel only for simplicity; could average
                self.ring_buf[self.ring_pos] = samples[i];
                self.ring_pos = (self.ring_pos + 1) % FFT_SIZE;
                i += channels;
            }
        }

        // Prepare FFT input: copy ring buffer to fft_buf with window, respecting circular order
        for idx in 0..FFT_SIZE {
            let ring_idx = (self.ring_pos + idx) % FFT_SIZE;
            let sample = self.ring_buf[ring_idx];
            self.fft_buf[idx].re = sample * self.window[idx];
            self.fft_buf[idx].im = 0.0;
        }

        // Run FFT (forward)
        self.fft.process(&mut self.fft_buf);

        // Accumulate power into bands
        let bin_count = FFT_SIZE / 2 + 1;
        let mut band_power = vec![0.0f32; BAND_COUNT];
        for bin in 0..bin_count {
            let re = self.fft_buf[bin].re;
            let im = self.fft_buf[bin].im;
            let power = re * re + im * im;
            let band = self.bin_to_band[bin] as usize;
            band_power[band] += power;
        }

        // Compute per‑band magnitude, smooth, and store atomically
        for band in 0..BAND_COUNT {
            // Normalize by number of bins to keep scaling comparable across bands
            let norm_factor = self.band_bin_count[band];
            let avg_power = if norm_factor > 0.0 {
                band_power[band] / norm_factor
            } else {
                0.0
            };
            let magnitude = avg_power.sqrt();
            self.smoothed[band] = SMOOTHING * self.smoothed[band] + (1.0 - SMOOTHING) * magnitude;
            // Write to shared output
            self.output.set_band(band, self.smoothed[band]);
        }
    }

    /// Retrieve the current smoothed spectrum (64 frequency bands).
    pub fn get_bands(&self) -> [f32; BAND_COUNT] {
        self.output.get_bands()
    }

    /// For backward compatibility: peak‑only magnitude (average of bands).
    pub fn get_magnitude(&self) -> f32 {
        // Approximate overall loudness as average of all bands
        let mut sum = 0.0;
        for &val in self.smoothed.iter() {
            sum += val;
        }
        sum / BAND_COUNT as f32
    }
}

impl Default for SpectrumAnalyzer {
    fn default() -> Self {
        // Default sample rate 44100 Hz – safe generic fallback
        Self::new(44100, Arc::new(SpectrumOutput::new()))
    }
}
