//! 音频解码器抽象层
//! 统一接口：Symphonia (FLAC/WAV/ALAC/APE/MP3...) + DSD (dsd-reader) + Rodio 回退

use std::fs::File;
use std::path::Path;

use rodio::Decoder as RodioDecoder;
use rodio::Source;

use crate::error::DecoderError;

/// 统一解码器 trait
pub trait AudioDecoder: Send {
    /// 采样率
    fn sample_rate(&self) -> u32;
    
    /// 声道数
    fn channels(&self) -> u16;
    
    /// 总时长 (秒)
    fn duration_secs(&self) -> f64;
    
    /// 解码下一批 PCM 样本 (交错 f32)
    /// 返回空 Vec 表示 EOF
    fn decode_next(&mut self) -> Result<Vec<f32>, DecoderError>;
    
    /// 跳转到指定样本位置 (可选，用于 seeking)
     fn seek(&mut self, _sample: u64) -> Result<(), DecoderError> {
         Err(DecoderError::SeekNotSupported)
     }
}

/// Rodio 原生解码器包装，作为 Symphonia 失败时的回退
/// 解码整个文件到内存中，适用于 Symphonia 无法处理的特殊格式文件
pub struct RodioDecoderWrapper {
    samples: Vec<f32>,
    position: usize,
    sample_rate: u32,
    channels: u16,
    duration_secs: f64,
}

impl RodioDecoderWrapper {
    pub fn open(path: &Path) -> Result<Self, DecoderError> {
        let file = File::open(path)
            .map_err(|e| DecoderError::IoError(e.to_string()))?;
        let decoder = RodioDecoder::new(file)
            .map_err(|e| DecoderError::DecoderError(format!("Rodio decode error: {}", e)))?;
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels() as u16;
        let duration_secs = decoder.total_duration()
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        let samples: Vec<f32> = decoder.convert_samples().collect();
        Ok(Self { samples, position: 0, sample_rate, channels, duration_secs })
    }
}

impl AudioDecoder for RodioDecoderWrapper {
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn channels(&self) -> u16 { self.channels }
    fn duration_secs(&self) -> f64 { self.duration_secs }

    fn decode_next(&mut self) -> Result<Vec<f32>, DecoderError> {
        const CHUNK_SIZE: usize = 4096;
        if self.position >= self.samples.len() {
            return Ok(Vec::new());
        }
        let end = (self.position + CHUNK_SIZE).min(self.samples.len());
        let chunk = self.samples[self.position..end].to_vec();
        self.position = end;
        Ok(chunk)
    }
}

/// 根据文件扩展名/内容自动选择解码器
/// 优先级: DSD > Symphonia > Rodio (回退)
pub fn open_decoder(path: &Path) -> Result<Box<dyn AudioDecoder>, DecoderError> {
    if let Some(ext) = path.extension() {
        let ext = ext.to_str().map(|s| s.to_lowercase()).unwrap_or_default();
        if ext == "dsf" || ext == "dff" {
            match crate::dsd_decoder::DsdDecoder::open(path) {
                Ok(dec) => return Ok(Box::new(dec)),
                Err(_) => {}
            }
        }
    }
    
    match crate::symphonia_decoder::SymphoniaDecoder::open(path) {
        Ok(dec) => return Ok(Box::new(dec)),
        Err(_) => {}
    }

    match RodioDecoderWrapper::open(path) {
        Ok(dec) => Ok(Box::new(dec)),
        Err(e) => Err(e),
    }
}
