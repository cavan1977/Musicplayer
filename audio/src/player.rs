//! Unified audio player that abstracts over backend implementations

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::backend::{AudioBackend, BackendEnum};
use crate::filter::FilterProfile;

/// Thread-safe, cloneable audio player handle
#[derive(Clone, Debug)]
pub struct Player {
    backend: Arc<Mutex<BackendEnum>>,
}

impl Player {
    /// Create a new player with the best available backend for the platform
    pub fn new() -> Self {
        let backend = BackendEnum::auto().expect("Failed to create audio backend");
        Self {
            backend: Arc::new(Mutex::new(backend)),
        }
    }

    /// Load and start playing a file from the beginning
    pub fn play<P: AsRef<Path>>(&self, path: P) -> Result<(), crate::error::PlaybackError> {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.load(path)?;
        backend.play()?;
        Ok(())
    }

    /// Pause playback
    pub fn pause(&self) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.pause();
    }

    /// Resume playback after pause
    pub fn resume(&self) -> Result<(), crate::error::PlaybackError> {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.play()
    }

    /// Stop playback and release resources
    pub fn stop(&self) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.stop();
    }

    /// Seek to a specific time in seconds
    pub fn seek(&self, seconds: f64) -> Result<(), crate::error::PlaybackError> {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.seek(seconds)
    }

    /// Get current playback state
    pub fn state(&self) -> crate::backend::PlaybackState {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.state()
    }

    /// Get current playback position in seconds
    pub fn progress_secs(&self) -> f64 {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.position_secs()
    }

    /// Get total duration of current track in seconds
    pub fn duration_secs(&self) -> f64 {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.duration_secs()
    }

    /// Get sample rate of current playback (0 if no file loaded)
    pub fn sample_rate(&self) -> u32 {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.sample_rate()
    }

    /// Set FIR filter profile (affects playback immediately if supported)
    pub fn set_filter(&self, profile: FilterProfile) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.set_filter(profile);
    }

    /// Set volume in decibels (0.0 = unity, negative = attenuation)
    pub fn set_volume_db(&self, db: f64) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.set_volume_db(db);
    }

     /// Increase volume by 0.01 dB
     pub fn volume_up(&self) {
         // Placeholder: actual implementation would adjust volume
         // e.g., let current = self.get_volume_db(); self.set_volume_db(current + 0.01);
     }

     /// Decrease volume by 0.01 dB
     pub fn volume_down(&self) {
         // Placeholder: actual implementation would adjust volume
     }

    /// Get current filter profile
    pub fn get_filter(&self) -> FilterProfile {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        match &*backend {
            BackendEnum::Rodio(b) => b.filter_profile,
            BackendEnum::Wasapi(b) => b.filter_profile(),
        }
    }

    /// Get current volume in dB
    pub fn get_volume_db(&self) -> f64 {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        match &*backend {
            BackendEnum::Rodio(b) => b.volume_db,
            BackendEnum::Wasapi(b) => b.volume_db(),
        }
    }

    /// Check if current sink is empty (no more samples to play)
    pub fn sink_empty(&self) -> bool {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        match &*backend {
            BackendEnum::Rodio(b) => b.sink_empty(),
            BackendEnum::Wasapi(b) => b.sink_empty(),
        }
    }

    /// Advance to the next track in the queue (gapless)
    pub fn advance_queue(&self) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        match &mut *backend {
            BackendEnum::Rodio(b) => b.advance_queue(),
            BackendEnum::Wasapi(b) => b.advance_queue(),
        }
    }

    /// Get current audio peak level (0.0 - 1.0)
    pub fn get_peak(&self) -> f32 {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        match &*backend {
            BackendEnum::Rodio(b) => b.get_peak(),
            BackendEnum::Wasapi(b) => b.get_peak(),
        }
    }

    /// Get current spectrum magnitude (0.0 - 1.0)
    pub fn get_spectrum(&self) -> f32 {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        match &*backend {
            BackendEnum::Rodio(b) => b.get_spectrum(),
            BackendEnum::Wasapi(b) => b.get_spectrum(),
        }
    }

    /// Get the current 64‑band frequency spectrum (each band 0.0‑1.0).
    pub fn get_spectrum_bands(&self) -> [f32; 64] {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        match &*backend {
            BackendEnum::Rodio(b) => b.get_spectrum_bands(),
            BackendEnum::Wasapi(b) => b.get_spectrum_bands(),
        }
    }

    /// Set playback mode (Sequential, LoopOne, LoopAll, Shuffle)
    pub fn set_playback_mode(&self, mode: crate::playback::PlaybackMode) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.set_playback_mode(mode);
    }

    /// Get current playback mode
    pub fn playback_mode(&self) -> crate::playback::PlaybackMode {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.playback_mode()
    }

    /// Set the playback queue (replaces current queue)
    pub fn set_queue(&self, paths: Vec<std::path::PathBuf>) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.set_queue(paths);
    }

    /// Add a track to the queue
    pub fn enqueue(&self, path: std::path::PathBuf) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.enqueue(path);
    }

    /// Get current queue
    pub fn get_queue(&self) -> Vec<std::path::PathBuf> {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.get_queue()
    }

    /// Clear the queue
    pub fn clear_queue(&self) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.clear_queue();
    }

    /// Set full playlist: original_queue gets all, queue gets songs after current_index
    pub fn set_playlist(&self, paths: Vec<std::path::PathBuf>, current_index: usize) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.set_playlist(paths, current_index);
    }

    /// Set audio output device by name (None = system default). Takes effect on next play.
    pub fn set_output_device(&self, device_name: Option<String>) {
        let mut backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.set_output_device(device_name);
    }

    /// Get current output device name
    pub fn output_device(&self) -> Option<String> {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.output_device()
    }

    /// List available audio output devices
    pub fn list_output_devices() -> Vec<String> {
        crate::device::list_output_devices()
    }

    /// Get the current file path being played
    pub fn current_file_path(&self) -> Option<PathBuf> {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        backend.current_file_path()
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
