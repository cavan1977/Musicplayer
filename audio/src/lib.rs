//! 音频播放模块
pub mod playback;
pub mod error;
pub mod backend;
pub mod player;
pub mod device;
pub mod decoder;
pub mod symphonia_decoder;
pub mod dsd_decoder;
pub mod filter;
pub mod volume;
pub mod pipeline;
pub mod wasapi_control;
pub mod wasapi_backend;
pub mod spectrum;  // FFT spectrum analyzer
pub mod metadata; // TrackMetadata for reading title/artist/album/duration

// Re-export the main player type at crate root
pub use player::Player;
pub use playback::PlaybackMode;
