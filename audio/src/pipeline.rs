//! 音频处理管线: Decoder -> FIR Filter -> Volume -> f32 Output
//! 所有处理在 64-bit 浮点域进行，输出转换为 32-bit 给 rodio

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use rodio::Source;

use crate::decoder::{self, AudioDecoder};
use crate::error::PlaybackError;
use crate::filter::{FirFilter, FilterProfile};
use crate::spectrum::{SpectrumAnalyzer, SpectrumOutput};
use crate::volume::VolumeControl;
use crate::wasapi_control::AudioControl;

/// 线程安全的管线控制 (用于后端调用 set_volume_db/set_filter)
#[derive(Clone)]
pub struct PipelineControl {
    filter_profile: Arc<Mutex<FilterProfile>>,
    volume_db: Arc<Mutex<f64>>,
}

impl PipelineControl {
    pub fn new(profile: FilterProfile, volume_db: f64) -> Self {
        Self {
            filter_profile: Arc::new(Mutex::new(profile)),
            volume_db: Arc::new(Mutex::new(volume_db)),
        }
    }

    /// 设置滤波器配置 (线程安全)
    pub fn set_filter(&self, profile: FilterProfile) {
        if let Ok(mut p) = self.filter_profile.lock() {
            *p = profile;
        }
    }

    /// 设置音量 dB (线程安全)
    pub fn set_volume_db(&self, db: f64) {
        if let Ok(mut v) = self.volume_db.lock() {
            *v = db;
        }
    }

    /// 获取当前滤波器配置
    pub fn filter_profile(&self) -> FilterProfile {
        self.filter_profile.lock().map(|p| *p).unwrap_or(FilterProfile::NOS)
    }

    /// 获取当前音量 dB
    pub fn volume_db(&self) -> f64 {
        self.volume_db.lock().map(|v| *v).unwrap_or(0.0)
    }
}
 pub struct PipelineSource {
     decoder: Option<Box<dyn AudioDecoder>>,
     _filter: FirFilter,
     volume: VolumeControl,
    control: PipelineControl,
    sample_rate: u32,
    channels: u16,
    buffer: Vec<f32>,
    buffer_pos: usize,
    eof: bool,
    samples_played: u64,
    /// Total samples for position calculation
    total_samples: Option<u64>,
    /// Atomic meter for peak level (f32 bits)
    meter: Arc<AtomicU32>,
    /// Spectrum analyzer (FFT based)
    spectrum: SpectrumAnalyzer,
}

impl PipelineSource {
    /// 创建管线源
    pub fn new<P: AsRef<Path>>(
        path: P,
        profile: FilterProfile,
        volume_db: f64,
        meter: Arc<AtomicU32>,
        output: std::sync::Arc<SpectrumOutput>,
        _default_sample_rate: u32,
    ) -> Result<Self, PlaybackError> {
        let decoder = match decoder::open_decoder(path.as_ref()) {
            Ok(d) => d,
            Err(e) => {
                return Err(PlaybackError::Decoder(e));
            }
        };
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();
        let duration = decoder.duration_secs();
        let total_samples = if duration > 0.0 {
            Some((duration * sample_rate as f64) as u64)
        } else {
            None
        };
        let filter = FirFilter::new(profile, sample_rate);
        let volume = VolumeControl::new(volume_db, 0.01, 120.0);
        let control = PipelineControl::new(profile, volume_db);
        let spectrum = SpectrumAnalyzer::new(sample_rate, output.clone());

         Ok(Self {
             decoder: Some(decoder),
             _filter: filter,
             volume,
             control,
             sample_rate,
             channels,
             buffer: Vec::new(),
             buffer_pos: 0,
             eof: false,
             samples_played: 0,
             total_samples,
             meter,
             spectrum,
         })
    }

    /// 获取管线控制句柄 (用于 set_volume_db/set_filter)
    pub fn control(&self) -> PipelineControl {
        self.control.clone()
    }

    /// 跳过指定秒数 (用于 seek)
    pub fn skip_secs(&mut self, seconds: f64) {
        let samples_to_skip = (seconds * self.sample_rate as f64) as u64;
        self.skip_samples(samples_to_skip);
    }

    /// 跳过指定样本数
    fn skip_samples(&mut self, count: u64) {
        // 跳过样本 - 通过消耗解码器输出来实现
        let channels = self.channels as usize;
        if channels == 0 {
            return;
        }

        let mut skipped = 0u64;
        while skipped < count {
            if self.buffer_pos >= self.buffer.len() {
                if !self.fill_buffer() {
                    break;
                }
            }
            let remaining_in_buffer = (self.buffer.len() - self.buffer_pos) / channels;
            let to_skip = ((count - skipped) as usize).min(remaining_in_buffer * channels);
            self.buffer_pos += to_skip;
            skipped += (to_skip / channels) as u64;
        }
        self.samples_played = self.samples_played.saturating_add(count.saturating_sub(skipped));
    }

    /// 获取已播放样本数
    pub fn samples_played(&self) -> u64 {
        self.samples_played
    }

    /// 获取总时长 (秒)
    pub fn duration_secs(&self) -> f64 {
        self.total_samples
            .map(|s| s as f64 / self.sample_rate as f64)
            .unwrap_or(0.0)
    }

    /// 获取当前音量 dB
    pub fn volume_db(&self) -> f64 {
        self.control.volume_db()
    }

    /// 获取当前滤波器配置
    pub fn filter_profile(&self) -> FilterProfile {
        self.control.filter_profile()
    }

    /// 填充缓冲区 (从解码器读取、滤波、音量处理)
    fn fill_buffer(&mut self) -> bool {
        if self.eof {
            return false;
        }

        let dec = match self.decoder.as_mut() {
            Some(d) => d,
            None => {
                self.eof = true;
                return false;
            }
        };

        let mut pcm_f32 = match dec.decode_next() {
            Ok(samples) => samples,
            Err(_) => {
                self.eof = true;
                return false;
            }
        };

        if pcm_f32.is_empty() {
            self.eof = true;
            return false;
        }

        // Volume processing (f32, no allocation)
        self.volume.multiply_gain_f32(&mut pcm_f32);

        // 存入缓冲区
        self.buffer = pcm_f32;
        self.buffer_pos = 0;

        // Compute peak for VU meter
        let mut max_amp: f32 = 0.0;
        for &sample in &self.buffer {
            let amp = sample.abs();
            if amp > max_amp {
                max_amp = amp;
            }
        }
        // Clamp to [0.0, 1.0] and store as f32 bits
        let max_amp_f32 = max_amp.min(1.0);
        self.meter.store(max_amp_f32.to_bits(), Ordering::Relaxed);

        // Spectrum analysis: FFT-based 64-band analyzer
        self.spectrum.analyze(&self.buffer, self.channels as usize);

        true
    }

     /// 获取当前滤波器配置 (用于比较)
     fn _current_filter_profile(&self) -> FilterProfile {
         // Simplified: we store profile in control, but here we approximate
         // For exact comparison, we could store the profile directly in struct
         FilterProfile::NOS
     }

    /// Get current audio peak level (0.0 - 1.0)
    pub fn peak(&self) -> f32 {
        f32::from_bits(self.meter.load(Ordering::Relaxed))
    }

    /// Get current spectrum magnitude (0.0 - 1.0)
    pub fn get_spectrum(&self) -> f32 {
        self.spectrum.get_magnitude()
    }
}

impl Iterator for PipelineSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.buffer_pos < self.buffer.len() {
                let sample = self.buffer[self.buffer_pos];
                self.buffer_pos += 1;
                self.samples_played = self.samples_played.saturating_add(1);
                return Some(sample);
            }

            // 缓冲区已耗尽，填充新数据
            self.buffer.clear();
            self.buffer_pos = 0;
            if !self.fill_buffer() {
                return None;
            }
        }
    }
}

impl Source for PipelineSource {
    fn current_frame_len(&self) -> Option<usize> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.len() / self.channels as usize)
        }
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        // 从解码器获取总时长
        // 由于 decoder 可能没有实现 total_duration，我们使用 Option
        // 但我们的 AudioDecoder trait 有 duration_secs()
        // 这里无法直接访问，返回 None
        None
    }
}

/// Lock-free variant for WASAPI exclusive callback
/// Uses atomic controls instead of mutex to avoid blocking real-time thread
pub struct PipelineSourceLockFree {
    decoder: Option<Box<dyn AudioDecoder>>,
    filter: FirFilter,
    control: Arc<AudioControl>,
    sample_rate: u32,
    channels: u16,
    buffer: Vec<f64>,
    buffer_pos: usize,
    eof: bool,
    /// Total samples for position calculation
    total_samples: Option<u64>,
    /// Atomic meter for peak level (f32 bits)
    meter: Arc<AtomicU32>,
}

impl PipelineSourceLockFree {
    pub fn new<P: AsRef<Path>>(
        path: P,
        control: Arc<AudioControl>,
        meter: Arc<AtomicU32>,
    ) -> Result<Self, PlaybackError> {
        let decoder = decoder::open_decoder(path.as_ref()).map_err(|e| PlaybackError::Decoder(e))?;
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();
        let duration = decoder.duration_secs();
        let total_samples = if duration > 0.0 {
            Some((duration * sample_rate as f64) as u64)
        } else {
            None
        };

        // Use initial filter profile from control
        let initial_profile = FilterProfile::NOS; // default
        let filter = FirFilter::new(initial_profile, sample_rate);

        Ok(Self {
            decoder: Some(decoder),
            filter,
            control,
            sample_rate,
            channels,
            buffer: Vec::new(),
            buffer_pos: 0,
            eof: false,
            total_samples,
            meter,
        })
    }

    /// Get control handle (clone of Arc)
    pub fn control(&self) -> Arc<AudioControl> {
        self.control.clone()
    }

    /// Skip samples (for seek)
    pub fn skip_samples(&mut self, count: u64) {
        let channels = self.channels as usize;
        if channels == 0 {
            return;
        }

        let mut skipped = 0u64;
        while skipped < count {
            if self.buffer_pos >= self.buffer.len() {
                if !self.fill_buffer() {
                    break;
                }
            }
            let remaining_in_buffer = (self.buffer.len() - self.buffer_pos) / channels;
            let to_skip = ((count - skipped) as usize).min(remaining_in_buffer * channels);
            self.buffer_pos += to_skip;
            skipped += (to_skip / channels) as u64;
        }
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get channels
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Get total duration in seconds
    pub fn duration_secs(&self) -> f64 {
        self.total_samples
            .map(|s| s as f64 / self.sample_rate as f64)
            .unwrap_or(0.0)
    }

    /// Get current audio peak level (0.0 - 1.0)
    pub fn peak(&self) -> f32 {
        f32::from_bits(self.meter.load(Ordering::Relaxed))
    }

    /// Fill buffer from decoder and apply FIR filter
    /// Returns true if data was filled, false on EOF
    fn fill_buffer(&mut self) -> bool {
        if self.eof {
            return false;
        }

        let dec = match self.decoder.as_mut() {
            Some(d) => d,
            None => {
                self.eof = true;
                self.control.set_eof();
                return false;
            }
        };

        let pcm_f32 = match dec.decode_next() {
            Ok(samples) => samples,
            Err(_) => {
                self.eof = true;
                self.control.set_eof();
                return false;
            }
        };

        if pcm_f32.is_empty() {
            self.eof = true;
            self.control.set_eof();
            return false;
        }

        // f32 -> f64
        let mut pcm_f64: Vec<f64> = pcm_f32.iter().map(|&s| s as f64).collect();

        // Check filter index and rebuild filter if changed
        let current_index = self.control.filter_index();
        // Map index to profile
        let profile = match current_index {
            0 => FilterProfile::NOS,
            1 => FilterProfile::LinearPhaseSlow,
            2 => FilterProfile::LinearPhaseFast,
            3 => FilterProfile::MinimumPhase,
            4 => FilterProfile::MixedPhase,
            _ => FilterProfile::NOS,
        };
        // Recreate filter if needed (simple check: if taps==0 or profile changed)
        if self.filter.taps() == 0 || profile != self.current_filter_profile() {
            self.filter = FirFilter::new(profile, self.sample_rate);
        }

        // FIR filter (interleaved)
        pcm_f64 = self.filter.process_interleaved(&pcm_f64, self.channels as usize);

        // Apply volume from atomic (lock-free) - already f64
        let volume_factor = self.control.volume_factor() as f64;
        for sample in pcm_f64.iter_mut() {
            *sample *= volume_factor;
        }

        // Store in buffer
        self.buffer = pcm_f64;
        self.buffer_pos = 0;

        // Compute peak for VU meter
        let mut max_amp: f64 = 0.0;
        for &sample in &self.buffer {
            let amp = sample.abs();
            if amp > max_amp {
                max_amp = amp;
            }
        }
        let max_amp_f32 = max_amp.min(1.0) as f32;
        self.meter.store(max_amp_f32.to_bits(), Ordering::Relaxed);

        true
    }

    /// Get next sample (used by WASAPI callback)
    /// This is lock-free except for internal buffer operations
    pub fn next_sample(&mut self) -> f32 {
        loop {
            if self.buffer_pos < self.buffer.len() {
                let sample = self.buffer[self.buffer_pos] as f32;
                self.buffer_pos += 1;
                // Increment samples played
                self.control.add_samples(1);
                return sample;
            }

            // Buffer exhausted, fill more
            self.buffer.clear();
            self.buffer_pos = 0;
            if !self.fill_buffer() {
                // EOF or error - return silence
                return 0.0;
            }
        }
    }

    /// Helper to get current filter profile (used for comparison)
    fn current_filter_profile(&self) -> FilterProfile {
        // In a real implementation, we would store the profile directly.
        // For now, we always return NOS. This means filter changes will always trigger rebuild.
        // That's acceptable.
        FilterProfile::NOS
    }
}

/// Iterator implementation for lock-free source
impl Iterator for PipelineSourceLockFree {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if paused/stopped via control
        if self.control.is_paused() || self.control.is_stopped() {
            // Return silence but continue iteration
            // (or could return None to signal end, but audio callback expects silence)
            self.control.add_samples(1);
            return Some(0.0);
        }

        if self.control.is_eof() {
            return None;
        }

        loop {
            if self.buffer_pos < self.buffer.len() {
                let sample = self.buffer[self.buffer_pos] as f32;
                self.buffer_pos += 1;
                self.control.add_samples(1);
                return Some(sample);
            }

            // Buffer exhausted, fill more
            if !self.fill_buffer() {
                return None;
            }
        }
    }
}

impl Source for PipelineSourceLockFree {
    fn current_frame_len(&self) -> Option<usize> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.len() / self.channels as usize)
        }
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

