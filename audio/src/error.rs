//! 音频模块错误定义

use std::fmt;

/// 播放器统一错误类型
#[derive(Debug, PartialEq)]
pub enum PlaybackError {
    Decoder(DecoderError),
    Device(DeviceError),
    NoDevice,
    Io(std::io::ErrorKind),
    Other(String),
}

impl fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaybackError::Decoder(e) => write!(f, "解码错误: {}", e),
            PlaybackError::Device(e) => write!(f, "设备错误: {}", e),
            PlaybackError::NoDevice => write!(f, "未找到音频设备"),
            PlaybackError::Io(kind) => write!(f, "IO 错误: {:?}", kind),
            PlaybackError::Other(msg) => write!(f, "其他错误: {}", msg),
        }
    }
}

impl std::error::Error for PlaybackError {}

/// 解码器错误
#[derive(Debug, PartialEq)]
pub enum DecoderError {
    IoError(String),
    FormatError(String),
    NoAudioTrack,
    MissingSampleRate,
    MissingChannels,
    DecoderError(String),
    Symphonia(String),
    UnsupportedSampleFormat,
    SeekNotSupported,
}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecoderError::IoError(msg) => write!(f, "IO 错误: {}", msg),
            DecoderError::FormatError(msg) => write!(f, "格式探测失败: {}", msg),
            DecoderError::NoAudioTrack => write!(f, "未找到音频轨道"),
            DecoderError::MissingSampleRate => write!(f, "缺少采样率信息"),
            DecoderError::MissingChannels => write!(f, "缺少声道信息"),
            DecoderError::DecoderError(msg) => write!(f, "解码器创建失败: {}", msg),
            DecoderError::Symphonia(msg) => write!(f, "Symphonia 错误: {}", msg),
            DecoderError::UnsupportedSampleFormat => write!(f, "不支持的采样格式"),
            DecoderError::SeekNotSupported => write!(f, "不支持跳转"),
        }
    }
}

impl std::error::Error for DecoderError {}

/// 设备错误
#[derive(Debug, PartialEq)]
pub enum DeviceError {
    NoDefaultDevice,
    UnsupportedFormat,
    StreamBuild(String),
    Play(String),
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceError::NoDefaultDevice => write!(f, "未找到默认音频设备"),
            DeviceError::UnsupportedFormat => write!(f, "不支持的音频格式"),
            DeviceError::StreamBuild(msg) => write!(f, "构建音频流失败: {}", msg),
            DeviceError::Play(msg) => write!(f, "播放错误: {}", msg),
        }
    }
}

impl std::error::Error for DeviceError {}

// From implementations (must be after type definitions)
impl From<DecoderError> for PlaybackError {
    fn from(err: DecoderError) -> Self {
        PlaybackError::Decoder(err)
    }
}

impl From<DeviceError> for PlaybackError {
    fn from(err: DeviceError) -> Self {
        PlaybackError::Device(err)
    }
}

impl From<std::io::Error> for PlaybackError {
    fn from(err: std::io::Error) -> Self {
        PlaybackError::Io(err.kind())
    }
}