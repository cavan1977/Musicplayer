//! DSD64 解码器 - 暂未实现
//! 后续将集成 dsd-reader 以支持 DSF/DFF 格式
use std::path::Path;

use crate::error::DecoderError;
use crate::decoder::AudioDecoder;

pub struct DsdDecoder;

impl AudioDecoder for DsdDecoder {
    fn sample_rate(&self) -> u32 { 0 }
    fn channels(&self) -> u16 { 0 }
    fn duration_secs(&self) -> f64 { 0.0 }
    fn decode_next(&mut self) -> Result<Vec<f32>, DecoderError> {
        Err(DecoderError::UnsupportedSampleFormat)
    }
    fn seek(&mut self, _sample: u64) -> Result<(), DecoderError> {
        Err(DecoderError::SeekNotSupported)
    }
}

impl DsdDecoder {
    pub fn open<P: AsRef<Path>>(_path: P) -> Result<Self, DecoderError> {
        Err(DecoderError::UnsupportedSampleFormat)
    }
}
