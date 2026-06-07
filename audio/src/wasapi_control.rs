//! Lock-free control primitives for WASAPI exclusive callback
//! Uses atomics to avoid blocking on real-time audio thread

use crate::filter::FilterProfile;
use crate::backend::PlaybackState;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

/// Linear volume factor stored as fixed-point (1000 = 1.0)
const VOLUME_SCALE: f32 = 1000.0;

#[derive(Debug)]
pub struct AudioControl {
    /// Volume factor * 1000 (0-1000 = 0.0-1.0)
    volume_factor: AtomicU32,
    /// Filter profile index (0=NOS, 1=LinearPhaseSlow, etc.)
    filter_index: AtomicU32,
    /// Playback state: 0=Stopped, 1=Playing, 2=Paused
    state: AtomicUsize,
    /// Total samples played (for position tracking)
    samples_played: AtomicU64,
    /// EOF flag (1 = end of stream)
    eof: AtomicUsize,
}

impl AudioControl {
    pub fn new() -> Self {
        Self {
            volume_factor: AtomicU32::new(1000), // unity
            filter_index: AtomicU32::new(0),     // NOS
            state: AtomicUsize::new(0),          // Stopped
            samples_played: AtomicU64::new(0),
            eof: AtomicUsize::new(0),
        }
    }

    /// Set volume in decibels (thread-safe)
    pub fn set_volume_db(&self, db: f64) {
        // Convert dB to linear: 0dB = 1.0, -60dB = 0.001
        let linear = 10.0_f64.powf(db / 20.0).clamp(0.0, 1.0);
        self.volume_factor.store((linear * VOLUME_SCALE as f64) as u32, Ordering::Release);
    }

    /// Get volume factor as f32 (lock-free for callback)
    #[inline]
    pub fn volume_factor(&self) -> f32 {
        self.volume_factor.load(Ordering::Acquire) as f32 / VOLUME_SCALE
    }

    /// Set filter profile (thread-safe)
    pub fn set_filter(&self, profile: FilterProfile) {
        let index = match profile {
            FilterProfile::NOS => 0,
            FilterProfile::LinearPhaseSlow => 1,
            FilterProfile::LinearPhaseFast => 2,
            FilterProfile::MinimumPhase => 3,
            FilterProfile::MixedPhase => 4,
        };
        self.filter_index.store(index, Ordering::Release);
    }

    /// Get filter index (for callback)
    #[inline]
    pub fn filter_index(&self) -> u32 {
        self.filter_index.load(Ordering::Acquire)
    }

    /// Set playback state (thread-safe)
    pub fn set_state(&self, state: PlaybackState) {
        let val = match state {
            PlaybackState::Stopped => 0,
            PlaybackState::Playing => 1,
            PlaybackState::Paused => 2,
        };
        self.state.store(val, Ordering::Release);
    }

    /// Check if currently playing (lock-free)
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.state.load(Ordering::Acquire) == 1
    }

    /// Check if paused (lock-free)
    #[inline]
    pub fn is_paused(&self) -> bool {
        self.state.load(Ordering::Acquire) == 2
    }

    /// Check if stopped (lock-free)
    #[inline]
    pub fn is_stopped(&self) -> bool {
        self.state.load(Ordering::Acquire) == 0
    }

    /// Increment samples played (called from callback)
    #[inline]
    pub fn add_samples(&self, count: u64) {
        self.samples_played.fetch_add(count, Ordering::Relaxed);
    }

    /// Get total samples played (for position)
    pub fn samples_played(&self) -> u64 {
        self.samples_played.load(Ordering::Acquire)
    }

    /// Mark end-of-stream
    pub fn set_eof(&self) {
        self.eof.store(1, Ordering::Release);
    }

    /// Check EOF flag
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.eof.load(Ordering::Acquire) == 1
    }

    /// Reset EOF (for new source)
    pub fn clear_eof(&self) {
        self.eof.store(0, Ordering::Release);
    }

    /// Reset samples played
    pub fn clear_samples(&self) {
        self.samples_played.store(0, Ordering::Release);
    }
}

impl Default for AudioControl {
    fn default() -> Self {
        Self::new()
    }
}
