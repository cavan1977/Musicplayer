//! Backend abstraction for audio playback

use std::path::Path;

use crate::error::PlaybackError;
use crate::filter::FilterProfile;

/// Backend-agnostic playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Audio backend trait
pub trait AudioBackend {
    /// Load a file and prepare for playback
    fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PlaybackError>;

    /// Start playback from current position or beginning
    fn play(&mut self) -> Result<(), PlaybackError>;

    /// Pause playback
    fn pause(&mut self);

    /// Stop playback and release resources
    fn stop(&mut self);

    /// Seek to position in seconds
    fn seek(&mut self, seconds: f64) -> Result<(), PlaybackError>;

    /// Get current playback state
    fn state(&self) -> PlaybackState;

    /// Get current position in seconds
    fn position_secs(&self) -> f64;

    /// Get total duration in seconds
    fn duration_secs(&self) -> f64;

    /// Get sample rate (for info/debug)
    fn sample_rate(&self) -> u32;

    /// Set volume in dB (0.0 = unity, negative = quieter)
    fn set_volume_db(&mut self, db: f64);

    /// Set filter profile
    fn set_filter(&mut self, profile: FilterProfile);

    /// Set playback mode (Sequential, LoopOne, LoopAll, Shuffle)
    fn set_playback_mode(&mut self, mode: crate::playback::PlaybackMode);

    /// Get current playback mode
    fn playback_mode(&self) -> crate::playback::PlaybackMode;

    /// Set the playback queue (replaces current queue)
    fn set_queue(&mut self, paths: Vec<std::path::PathBuf>);

    /// Add a track to the queue
    fn enqueue(&mut self, path: std::path::PathBuf);

    /// Get current queue
    fn get_queue(&self) -> Vec<std::path::PathBuf>;

    /// Clear the queue
    fn clear_queue(&mut self);

    /// Set full playlist: original_queue = all, queue = songs after current_index
    fn set_playlist(&mut self, paths: Vec<std::path::PathBuf>, current_index: usize);

    /// Set audio output device by name (None = system default)
    fn set_output_device(&mut self, device_name: Option<String>);

    /// Get current output device name
    fn output_device(&self) -> Option<String>;

    /// Get current file path being played
    fn current_file_path(&self) -> Option<std::path::PathBuf>;
}

/// Rodio backend adapter
#[derive(Debug)]
pub struct RodioBackend {
    player: crate::playback::Player,
    pub filter_profile: FilterProfile,
    pub volume_db: f64,
}

impl RodioBackend {
    pub fn new() -> Result<Self, PlaybackError> {
        Ok(Self { 
            player: crate::playback::Player::new(),
            filter_profile: FilterProfile::NOS,
            volume_db: 0.0,
        })
    }

    /// Check if current sink is empty (no more samples to play)
    pub fn sink_empty(&self) -> bool {
        self.player.sink_empty()
    }

    /// Advance to the next track in the queue (gapless)
    pub fn advance_queue(&self) {
        self.player.advance_queue()
    }

    /// Get current audio peak level (0.0 - 1.0)
    pub fn get_peak(&self) -> f32 {
        self.player.get_peak()
    }

    /// Get current spectrum magnitude (0.0 - 1.0)
    pub fn get_spectrum(&self) -> f32 {
        self.player.get_magnitude() // delegate to new method
    }

    /// Get the current 64‑band frequency spectrum.
    pub fn get_spectrum_bands(&self) -> [f32; 64] {
        self.player.get_spectrum_bands()
    }
}

impl AudioBackend for RodioBackend {
    fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PlaybackError> {
        self.player.play(path)
    }

    fn play(&mut self) -> Result<(), PlaybackError> {
        self.player.resume();
        Ok(())
    }

    fn pause(&mut self) {
        self.player.pause();
    }

    fn stop(&mut self) {
        self.player.stop();
    }

    fn seek(&mut self, seconds: f64) -> Result<(), PlaybackError> {
        self.player.seek(seconds)
    }

    fn state(&self) -> PlaybackState {
        match self.player.state() {
            crate::playback::PlaybackState::Stopped => PlaybackState::Stopped,
            crate::playback::PlaybackState::Playing => PlaybackState::Playing,
            crate::playback::PlaybackState::Paused => PlaybackState::Paused,
        }
    }

    fn position_secs(&self) -> f64 {
        self.player.progress_secs()
    }

    fn duration_secs(&self) -> f64 {
        self.player.duration_secs()
    }

    fn sample_rate(&self) -> u32 {
        self.player.sample_rate()
    }

    fn set_volume_db(&mut self, db: f64) {
        self.volume_db = db;
        self.player.set_volume_db(db);
    }

    fn set_filter(&mut self, profile: FilterProfile) {
        self.filter_profile = profile;
        self.player.set_filter(profile);
    }

    fn set_playback_mode(&mut self, mode: crate::playback::PlaybackMode) {
        self.player.set_playback_mode(mode);
    }

    fn playback_mode(&self) -> crate::playback::PlaybackMode {
        self.player.playback_mode()
    }

    fn set_queue(&mut self, paths: Vec<std::path::PathBuf>) {
        self.player.set_queue(paths);
    }

    fn enqueue(&mut self, path: std::path::PathBuf) {
        self.player.enqueue(path);
    }

    fn get_queue(&self) -> Vec<std::path::PathBuf> {
        self.player.get_queue()
    }

    fn clear_queue(&mut self) {
        self.player.clear_queue();
    }

    fn set_playlist(&mut self, paths: Vec<std::path::PathBuf>, current_index: usize) {
        self.player.set_playlist(paths, current_index);
    }

    fn set_output_device(&mut self, device_name: Option<String>) {
        self.player.set_output_device(device_name);
    }

    fn output_device(&self) -> Option<String> {
        self.player.output_device()
    }

    fn current_file_path(&self) -> Option<std::path::PathBuf> {
        self.player.current_file_path()
    }
}

/// WASAPI backend adapter (true exclusive mode via cpal)
#[derive(Debug)]
pub struct WasapiBackend {
    backend: crate::wasapi_backend::WasapiBackend,
}

impl WasapiBackend {
    pub fn new() -> Result<Self, PlaybackError> {
        Ok(Self { 
            backend: crate::wasapi_backend::WasapiBackend::new()? 
        })
    }

    /// Get current filter profile
    pub fn filter_profile(&self) -> FilterProfile {
        self.backend.filter_profile
    }

    /// Get current volume in dB
    pub fn volume_db(&self) -> f64 {
        self.backend.volume_db
    }

    /// Check if current sink is empty (no more samples to play)
    pub fn sink_empty(&self) -> bool {
        self.backend.sink_empty()
    }

    /// Advance to the next track in the queue (gapless)
    pub fn advance_queue(&mut self) {
        self.backend.advance_queue()
    }

    /// Get current audio peak level (0.0 - 1.0)
    pub fn get_peak(&self) -> f32 {
        self.backend.get_peak()
    }

    /// Get current spectrum magnitude (0.0 - 1.0)
    pub fn get_spectrum(&self) -> f32 {
        // WASAPI doesn't currently expose spectrum
        0.0
    }

    /// Get the current 64‑band frequency spectrum.
    pub fn get_spectrum_bands(&self) -> [f32; 64] {
        // Placeholder – not implemented for WASAPI backend yet
        [0.0f32; 64]
    }
}

impl AudioBackend for WasapiBackend {
    fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PlaybackError> {
        self.backend.load(path)
    }

    fn play(&mut self) -> Result<(), PlaybackError> {
        self.backend.play()
    }

    fn pause(&mut self) {
        self.backend.pause();
    }

    fn stop(&mut self) {
        self.backend.stop();
    }

    fn seek(&mut self, seconds: f64) -> Result<(), PlaybackError> {
        self.backend.seek(seconds)
    }

    fn state(&self) -> PlaybackState {
        self.backend.state()
    }

    fn position_secs(&self) -> f64 {
        self.backend.position_secs()
    }

    fn duration_secs(&self) -> f64 {
        self.backend.duration_secs()
    }

    fn sample_rate(&self) -> u32 {
        self.backend.sample_rate()
    }

    fn set_volume_db(&mut self, db: f64) {
        self.backend.set_volume_db(db);
    }

    fn set_filter(&mut self, profile: FilterProfile) {
        self.backend.set_filter(profile);
    }

    fn set_playback_mode(&mut self, mode: crate::playback::PlaybackMode) {
        self.backend.set_playback_mode(mode);
    }

    fn playback_mode(&self) -> crate::playback::PlaybackMode {
        self.backend.playback_mode()
    }

    fn set_queue(&mut self, paths: Vec<std::path::PathBuf>) {
        self.backend.set_queue(paths);
    }

    fn enqueue(&mut self, path: std::path::PathBuf) {
        self.backend.enqueue(path);
    }

    fn get_queue(&self) -> Vec<std::path::PathBuf> {
        self.backend.get_queue()
    }

    fn clear_queue(&mut self) {
        self.backend.clear_queue();
    }

    fn set_playlist(&mut self, paths: Vec<std::path::PathBuf>, current_index: usize) {
        self.backend.set_playlist(paths, current_index);
    }

    fn set_output_device(&mut self, device_name: Option<String>) {
        self.backend.set_output_device(device_name);
    }

    fn output_device(&self) -> Option<String> {
        self.backend.output_device()
    }

    fn current_file_path(&self) -> Option<std::path::PathBuf> {
        self.backend.current_file_path()
    }
}

/// Enum selecting which backend to use
#[derive(Debug)]
pub enum BackendEnum {
    Rodio(RodioBackend),
    Wasapi(WasapiBackend),
}

impl BackendEnum {
    /// Create a Rodio backend
    pub fn rodio() -> Result<Self, PlaybackError> {
        Ok(Self::Rodio(RodioBackend::new()?))
    }

    /// Create a WASAPI backend (Windows only)
    #[cfg(target_os = "windows")]
    pub fn wasapi() -> Result<Self, PlaybackError> {
        Ok(Self::Wasapi(WasapiBackend::new()?))
    }

    /// Auto-select best available backend
    pub fn auto() -> Result<Self, PlaybackError> {
        // Use Rodio by default for better device compatibility
        // WASAPI exclusive mode requires exact sample rate matching
        Self::rodio()
    }
}

impl AudioBackend for BackendEnum {
    fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PlaybackError> {
        match self {
            BackendEnum::Rodio(b) => b.load(path),
            BackendEnum::Wasapi(b) => b.load(path),
        }
    }

    fn play(&mut self) -> Result<(), PlaybackError> {
        match self {
            BackendEnum::Rodio(b) => b.play(),
            BackendEnum::Wasapi(b) => b.play(),
        }
    }

    fn pause(&mut self) {
        match self {
            BackendEnum::Rodio(b) => b.pause(),
            BackendEnum::Wasapi(b) => b.pause(),
        }
    }

    fn stop(&mut self) {
        match self {
            BackendEnum::Rodio(b) => b.stop(),
            BackendEnum::Wasapi(b) => b.stop(),
        }
    }

    fn seek(&mut self, seconds: f64) -> Result<(), PlaybackError> {
        match self {
            BackendEnum::Rodio(b) => b.seek(seconds),
            BackendEnum::Wasapi(b) => b.seek(seconds),
        }
    }

    fn state(&self) -> PlaybackState {
        match self {
            BackendEnum::Rodio(b) => b.state(),
            BackendEnum::Wasapi(b) => b.state(),
        }
    }

    fn position_secs(&self) -> f64 {
        match self {
            BackendEnum::Rodio(b) => b.position_secs(),
            BackendEnum::Wasapi(b) => b.position_secs(),
        }
    }

    fn duration_secs(&self) -> f64 {
        match self {
            BackendEnum::Rodio(b) => b.duration_secs(),
            BackendEnum::Wasapi(b) => b.duration_secs(),
        }
    }

    fn sample_rate(&self) -> u32 {
        match self {
            BackendEnum::Rodio(b) => b.sample_rate(),
            BackendEnum::Wasapi(b) => b.sample_rate(),
        }
    }

    fn set_volume_db(&mut self, db: f64) {
        match self {
            BackendEnum::Rodio(b) => b.set_volume_db(db),
            BackendEnum::Wasapi(b) => b.set_volume_db(db),
        }
    }

    fn set_filter(&mut self, profile: FilterProfile) {
        match self {
            BackendEnum::Rodio(b) => b.set_filter(profile),
            BackendEnum::Wasapi(b) => b.set_filter(profile),
        }
    }

    fn set_playback_mode(&mut self, mode: crate::playback::PlaybackMode) {
        match self {
            BackendEnum::Rodio(b) => b.set_playback_mode(mode),
            BackendEnum::Wasapi(b) => b.set_playback_mode(mode),
        }
    }

    fn playback_mode(&self) -> crate::playback::PlaybackMode {
        match self {
            BackendEnum::Rodio(b) => b.playback_mode(),
            BackendEnum::Wasapi(b) => b.playback_mode(),
        }
    }

    fn set_queue(&mut self, paths: Vec<std::path::PathBuf>) {
        match self {
            BackendEnum::Rodio(b) => b.set_queue(paths),
            BackendEnum::Wasapi(b) => b.set_queue(paths),
        }
    }

    fn enqueue(&mut self, path: std::path::PathBuf) {
        match self {
            BackendEnum::Rodio(b) => b.enqueue(path),
            BackendEnum::Wasapi(b) => b.enqueue(path),
        }
    }

    fn get_queue(&self) -> Vec<std::path::PathBuf> {
        match self {
            BackendEnum::Rodio(b) => b.get_queue(),
            BackendEnum::Wasapi(b) => b.get_queue(),
        }
    }

    fn clear_queue(&mut self) {
        match self {
            BackendEnum::Rodio(b) => b.clear_queue(),
            BackendEnum::Wasapi(b) => b.clear_queue(),
        }
    }

    fn set_playlist(&mut self, paths: Vec<std::path::PathBuf>, current_index: usize) {
        match self {
            BackendEnum::Rodio(b) => b.set_playlist(paths, current_index),
            BackendEnum::Wasapi(b) => b.set_playlist(paths, current_index),
        }
    }

    fn set_output_device(&mut self, device_name: Option<String>) {
        match self {
            BackendEnum::Rodio(b) => b.set_output_device(device_name),
            BackendEnum::Wasapi(b) => b.set_output_device(device_name),
        }
    }

    fn output_device(&self) -> Option<String> {
        match self {
            BackendEnum::Rodio(b) => b.output_device(),
            BackendEnum::Wasapi(b) => b.output_device(),
        }
    }

    fn current_file_path(&self) -> Option<std::path::PathBuf> {
        match self {
            BackendEnum::Rodio(b) => b.current_file_path(),
            BackendEnum::Wasapi(b) => b.current_file_path(),
        }
    }
}
