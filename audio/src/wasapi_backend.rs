//! WASAPI exclusive mode backend using cpal
//! Bit-perfect playback with lock-free control

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use std::fmt;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig, SampleRate};

use super::error::{PlaybackError, DeviceError};
use super::backend::AudioBackend;
use super::backend::PlaybackState;
use super::filter::FilterProfile;
use super::pipeline::PipelineSourceLockFree;
use super::wasapi_control::AudioControl;
use super::playback::PlaybackMode;

/// WASAPI exclusive backend - bit-perfect, no resampling
pub struct WasapiBackend {
    /// Windows audio output device
    device: Device,
    /// Active output stream (if playing/paused)
    stream: Option<Stream>,
    /// Lock-free control shared with callback
    control: Arc<AudioControl>,
    /// Current audio source (lock-free pipeline)
    source: Option<PipelineSourceLockFree>,
    /// Stored file path for seek/reload
    file_path: Option<PathBuf>,
    /// Current filter profile setting
    pub filter_profile: FilterProfile,
    /// Current volume dB setting
    pub volume_db: f64,
    /// Sample rate (from device and file)
    sample_rate: u32,
    /// Channels (stereo = 2)
    channels: u16,
    /// Duration in seconds (from file metadata)
    duration: f64,
    /// Playback state (mirrors control.state but for UI)
    state: PlaybackState,
    /// When playback started
    play_start: Option<Instant>,
    /// Total paused duration
    total_paused: Duration,
    /// When pause was initiated
    pause_start: Option<Instant>,
    /// Starting offset in samples (for seek)
    start_offset_samples: u64,
    /// Gapless playback queue
    queue: Vec<PathBuf>,
    /// Original queue order (for shuffle)
    original_queue: Vec<PathBuf>,
    /// Current playback mode
    mode: super::playback::PlaybackMode,
    /// Selected output device name (None = system default)
    output_device: Option<String>,
    /// Atomic meter for peak level (shared with pipeline)
    meter: Option<Arc<AtomicU32>>,
}

impl WasapiBackend {
    /// Create new WASAPI backend with default device
    pub fn new() -> Result<Self, PlaybackError> {
        #[cfg(target_os = "windows")]
        {
            use cpal::default_host;

            let host = default_host();
            let device = host.default_output_device()
                .ok_or(PlaybackError::Device(DeviceError::NoDefaultDevice))?;

            // Query default config to get sample rate/channels
            let supported_config = device.default_output_config()
                .map_err(|e| PlaybackError::Device(DeviceError::StreamBuild(e.to_string())))?;

            let sample_rate = supported_config.sample_rate().0;
            let channels = supported_config.channels();

            Ok(Self {
                device,
                stream: None,
                control: Arc::new(AudioControl::new()),
                source: None,
                file_path: None,
                filter_profile: FilterProfile::NOS,
                volume_db: 0.0,
                sample_rate,
                channels,
                duration: 0.0,
                state: PlaybackState::Stopped,
                play_start: None,
                total_paused: Duration::ZERO,
                pause_start: None,
                start_offset_samples: 0,
                queue: Vec::new(),
                original_queue: Vec::new(),
                mode: super::playback::PlaybackMode::Sequential,
                output_device: None,
                meter: None,
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(PlaybackError::Device(DeviceError::NoDefaultDevice))
        }
    }

    /// Verify that the given sample rate matches the device exactly
    fn validate_sample_rate(&self, required_rate: u32) -> Result<(), PlaybackError> {
        if required_rate != self.sample_rate {
            return Err(PlaybackError::Device(DeviceError::UnsupportedFormat));
        }
        Ok(())
    }

    /// Build the output stream (cpal) and start playback
    fn build_and_start_stream(&mut self) -> Result<(), PlaybackError> {
        // Take the source out of self
        let source = self.source.take()
            .ok_or_else(|| PlaybackError::Other("No audio source loaded".into()))?;

        // Ensure sample rate matches device (exclusive mode requirement)
        self.validate_sample_rate(source.sample_rate())?;

        let config = StreamConfig {
            channels: source.channels(),
            sample_rate: SampleRate(source.sample_rate()),
            buffer_size: cpal::BufferSize::Default,
        };

        // Clone control for callback
        let control = self.control.clone();

        // For the callback, we need a mutable reference to the source.
        // This is tricky: callback runs on audio thread and we cannot share &mut.
        // Use Arc<Mutex> to hold source. The callback locks very briefly.
        let source_arc = Arc::new(std::sync::Mutex::new(source));

        let stream = self.device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Fast lock (<1ms) to get samples
                let mut src_guard = match source_arc.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        data.fill(0.0);
                        return;
                    }
                };
                let src = &mut *src_guard;

                // Check control state atomically (no lock)
                if !control.is_playing() {
                    data.fill(0.0);
                    return;
                }

                if control.is_eof() {
                    data.fill(0.0);
                    return;
                }

                // Fill output buffer
                for sample_out in data.iter_mut() {
                    *sample_out = src.next_sample();
                }
            },
            |err| eprintln!("WASAPI callback error: {}", err),
            None,
        ).map_err(|e: cpal::BuildStreamError| PlaybackError::Device(DeviceError::StreamBuild(e.to_string())))?;

        // Start stream
        stream.play().map_err(|e| PlaybackError::Device(DeviceError::Play(e.to_string())))?;

        // Store stream and move source_arc back into self.source if we need to access later.
        // For now, we keep it in the closure; for seek we'll drop and recreate.
        self.stream = Some(stream);
        // We no longer have self.source; it's moved into closure. Set to None.
        self.source = None;

        Ok(())
    }

    /// Check if current sink is empty (no more samples to play)
    pub fn sink_empty(&self) -> bool {
        self.state == PlaybackState::Playing && self.control.is_eof()
    }

    /// Advance to the next track based on playback mode (gapless)
    pub fn advance_queue(&mut self) {
        if self.state != PlaybackState::Playing {
            return;
        }

        // Determine next path based on mode (FIFO)
        let pop_front = |queue: &mut Vec<PathBuf>| -> Option<PathBuf> {
            if queue.is_empty() { None } else { Some(queue.remove(0)) }
        };

        let next_path = match self.mode {
            PlaybackMode::Sequential => pop_front(&mut self.queue),
            PlaybackMode::LoopOne => self.file_path.clone(),
            PlaybackMode::LoopAll => {
                if self.queue.is_empty() {
                    self.queue = self.original_queue.clone();
                }
                pop_front(&mut self.queue)
            }
            PlaybackMode::Shuffle => {
                if self.queue.is_empty() {
                    use rand::seq::SliceRandom;
                    let mut rng = rand::thread_rng();
                    let mut shuffled = self.original_queue.clone();
                    if let Some(current) = &self.file_path {
                        shuffled.retain(|p| p != current);
                    }
                    shuffled.shuffle(&mut rng);
                    self.queue = shuffled;
                }
                pop_front(&mut self.queue)
            }
        };

        let next_path = match next_path {
            Some(p) => p,
            None => {
                // No more tracks: stop playback and reset state
                self.stop();
                // Clear metadata so duration becomes 0
                self.file_path = None;
                self.duration = 0.0;
                self.sample_rate = 0;
                return;
            }
        };

        // Stop current playback completely
        self.stop();
        // Load and start next track
        if let Err(e) = self.load(&next_path) {
            eprintln!("Failed to advance to {}: {}", next_path.display(), e);
        }
    }

    /// Set playback mode
    pub fn set_playback_mode(&mut self, mode: PlaybackMode) {
        self.mode = mode;
        // If switching to Shuffle, reshuffle queue immediately
        if mode == PlaybackMode::Shuffle && !self.original_queue.is_empty() {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            let mut shuffled = self.original_queue.clone();
            if let Some(current) = &self.file_path {
                shuffled.retain(|p| p != current);
            }
            shuffled.shuffle(&mut rng);
            self.queue = shuffled;
        }
    }

    /// Get current playback mode
    pub fn playback_mode(&self) -> PlaybackMode {
        self.mode
    }

    /// Set the playback queue (replaces current queue)
    pub fn set_queue(&mut self, paths: Vec<PathBuf>) {
        self.queue = paths.clone();
        self.original_queue = paths;
    }

    /// Add a track to the queue
    pub fn enqueue(&mut self, path: PathBuf) {
        self.queue.push(path.clone());
        self.original_queue.push(path);
    }

    /// Get current queue
    pub fn get_queue(&self) -> Vec<PathBuf> {
        self.queue.clone()
    }

    /// Clear the queue
    pub fn clear_queue(&mut self) {
        self.queue.clear();
        self.original_queue.clear();
    }

    /// Set full playlist: original_queue = all paths, queue = songs after current_index
    pub fn set_playlist(&mut self, paths: Vec<PathBuf>, current_index: usize) {
        self.original_queue = paths.clone();
        if current_index + 1 < paths.len() {
            self.queue = paths[current_index + 1..].to_vec();
        } else {
            self.queue.clear();
        }
    }

    /// Set audio output device by name
    pub fn set_output_device(&mut self, device_name: Option<String>) {
        self.output_device = device_name;
    }

    /// Get current output device name
    pub fn output_device(&self) -> Option<String> {
        self.output_device.clone()
    }

    /// Get current audio peak level (0.0 - 1.0)
    pub fn get_peak(&self) -> f32 {
        match &self.meter {
            Some(meter) => f32::from_bits(meter.load(Ordering::Relaxed)),
            None => 0.0,
        }
    }
}

impl AudioBackend for WasapiBackend {
    fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PlaybackError> {
        // If already playing/paused, enqueue the track instead of loading immediately
        if self.stream.is_some() {
            self.queue.push(path.as_ref().to_path_buf());
            return Ok(());
        }

        // If stopped, clear queue to match fresh play behavior
        self.queue.clear();

        // Stop any existing playback (drop stream)
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        let path_ref = path.as_ref();
        self.file_path = Some(path_ref.to_path_buf());

        // Reset control
        self.control = Arc::new(AudioControl::new());
        self.control.set_volume_db(self.volume_db);
        self.control.set_filter(self.filter_profile);
        self.control.clear_eof();
        self.control.clear_samples();

        // Create lock-free pipeline source with VU meter
        let meter = Arc::new(AtomicU32::new(0));
        let mut source = PipelineSourceLockFree::new(path_ref, self.control.clone(), meter.clone())
            .map_err(|e| PlaybackError::Other(e.to_string()))?;
        self.meter = Some(meter);

        // Apply initial seek offset if any
        if self.start_offset_samples > 0 {
            source.skip_samples(self.start_offset_samples);
            self.start_offset_samples = 0;
        }

        self.sample_rate = source.sample_rate();
        self.channels = source.channels();
        self.duration = source.duration_secs();
        self.source = Some(source);

        // Start stream immediately (playing)
        self.control.set_state(PlaybackState::Playing);
        self.build_and_start_stream()?;

        self.state = PlaybackState::Playing;
        self.play_start = Some(Instant::now());
        self.total_paused = Duration::ZERO;
        self.pause_start = None;

        Ok(())
    }

    fn play(&mut self) -> Result<(), PlaybackError> {
        match self.state {
            PlaybackState::Paused => {
                // Resume: update control, stream already alive
                self.control.set_state(PlaybackState::Playing);
                if let Some(pause_start) = self.pause_start {
                    self.total_paused += pause_start.elapsed();
                }
                self.pause_start = None;
                self.play_start = Some(Instant::now());
                self.state = PlaybackState::Playing;
                Ok(())
            }
            PlaybackState::Stopped => {
                // Need to reload file first
                if let Some(path) = self.file_path.clone() {
                    self.load(path)?;
                    Ok(())
                } else {
                    Err(PlaybackError::Other("No file to play".into()))
                }
            }
            PlaybackState::Playing => Ok(()),
        }
    }

    fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            // Just update state - stream stays alive
            self.control.set_state(PlaybackState::Paused);
            self.pause_start = Some(Instant::now());
            self.state = PlaybackState::Paused;
        }
    }

    fn stop(&mut self) {
        // Drop stream completely
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        self.source = None;
        self.meter = None;
        self.control.set_state(PlaybackState::Stopped);
        self.state = PlaybackState::Stopped;
        self.play_start = None;
        self.pause_start = None;
        self.total_paused = Duration::ZERO;
        self.file_path = None;
        self.duration = 0.0;
        self.start_offset_samples = 0;
    }

    fn seek(&mut self, seconds: f64) -> Result<(), PlaybackError> {
        let offset_samples = (seconds * self.sample_rate as f64) as u64;

        // Stop stream (drop)
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        // Store offset for next load
        self.start_offset_samples = offset_samples;

        // Reload from same file with offset (clone path to avoid borrow conflict)
        let path_opt = self.file_path.clone();
        match path_opt {
            Some(path) => {
                let was_playing = self.state == PlaybackState::Playing;
                self.load(path)?;
                if !was_playing {
                    self.pause();
                }
                Ok(())
            }
            None => Err(PlaybackError::Other("No file loaded for seek".into())),
        }
    }

    fn set_volume_db(&mut self, db: f64) {
        self.volume_db = db;
        self.control.set_volume_db(db);
    }

    fn set_filter(&mut self, profile: FilterProfile) {
        self.filter_profile = profile;
    }

    fn set_playback_mode(&mut self, mode: crate::playback::PlaybackMode) {
        self.mode = mode;
        // If switching to Shuffle, reshuffle queue immediately
        if mode == crate::playback::PlaybackMode::Shuffle && !self.original_queue.is_empty() {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            let mut shuffled = self.original_queue.clone();
            if let Some(current) = &self.file_path {
                shuffled.retain(|p| p != current);
            }
            shuffled.shuffle(&mut rng);
            self.queue = shuffled;
        }
    }

    fn playback_mode(&self) -> crate::playback::PlaybackMode {
        self.mode
    }

    fn set_queue(&mut self, paths: Vec<std::path::PathBuf>) {
        self.queue = paths.clone();
        self.original_queue = paths;
    }

    fn enqueue(&mut self, path: std::path::PathBuf) {
        self.queue.push(path.clone());
        self.original_queue.push(path);
    }

    fn get_queue(&self) -> Vec<std::path::PathBuf> {
        self.queue.clone()
    }

    fn clear_queue(&mut self) {
        self.queue.clear();
        self.original_queue.clear();
    }

    fn set_playlist(&mut self, paths: Vec<std::path::PathBuf>, current_index: usize) {
        self.original_queue = paths.clone();
        if current_index + 1 < paths.len() {
            self.queue = paths[current_index + 1..].to_vec();
        } else {
            self.queue.clear();
        }
    }

    fn set_output_device(&mut self, device_name: Option<String>) {
        self.output_device = device_name;
    }

    fn output_device(&self) -> Option<String> {
        self.output_device.clone()
    }

    fn state(&self) -> PlaybackState {
        self.state
    }

    fn position_secs(&self) -> f64 {
        let played_samples = self.control.samples_played();
        if let Some(start_offset) = self.start_offset_samples.checked_sub(played_samples) {
            // If we haven't passed the start offset yet (seeking forward)
            return start_offset as f64 / self.sample_rate as f64;
        }
        let net_samples = played_samples.saturating_sub(self.start_offset_samples);
        let elapsed_secs = net_samples as f64 / self.sample_rate as f64;

        match self.state {
            PlaybackState::Playing => {
                if let Some(start) = self.play_start {
                    let wall_elapsed = start.elapsed().as_secs_f64();
                    // Wall time may differ from sample time slightly
                    return (wall_elapsed - self.total_paused.as_secs_f64()).max(0.0);
                }
                elapsed_secs
            }
            PlaybackState::Paused => {
                // When paused, samples counter stops, so we need to compute from when play started
                if let Some(start) = self.play_start {
                    let wall_elapsed = start.elapsed().as_secs_f64();
                    let paused_elapsed = self.pause_start.map(|p| p.elapsed().as_secs_f64()).unwrap_or(0.0);
                    return (wall_elapsed - self.total_paused.as_secs_f64() - paused_elapsed).max(0.0);
                }
                elapsed_secs
            }
            PlaybackState::Stopped => 0.0,
        }
    }

    fn duration_secs(&self) -> f64 {
        self.duration
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn current_file_path(&self) -> Option<PathBuf> {
        self.file_path.clone()
    }
}

impl fmt::Debug for WasapiBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WasapiBackend")
            .field("state", &self.state)
            .field("sample_rate", &self.sample_rate)
            .field("channels", &self.channels)
            .field("filter_profile", &self.filter_profile)
            .field("volume_db", &self.volume_db)
            .field("duration", &self.duration)
            .field("file_path", &self.file_path)
            .finish()
    }
}

impl Default for WasapiBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create WASAPI backend")
    }
}
