//! Symphonia 解码器实现 - 直接使用 symphonia 核心
//! 支持 FLAC, WAV, ALAC, MP3 等格式
use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::DecoderError;
use crate::decoder::AudioDecoder;

pub struct SymphoniaDecoder {
    sample_rate: u32,
    channels: u16,
    duration_secs: f64,
    // 使用 symphonia 格式探测和解码
    format: Box<dyn symphonia::core::formats::FormatReader>,
    track: u32,
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    current_sample: u64,
    // 样本缓冲区
    sample_buf: Option<SampleBuffer<f32>>,
    consumed_samples: usize,
}

impl SymphoniaDecoder {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DecoderError> {
        let path_r = path.as_ref();

        let file = File::open(&path)
            .map_err(|e| DecoderError::IoError(e.to_string()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path_r.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "flac" => { hint.with_extension("flac"); },
                "mp3" => { hint.with_extension("mp3"); },
                "wav" => { hint.with_extension("wav"); },
                "ogg" => { hint.with_extension("ogg"); },
                "m4a" => { hint.with_extension("m4a"); },
                "aac" => { hint.with_extension("aac"); },
                "aiff" | "aif" => { hint.with_extension("aiff"); },
                "wma" => { hint.with_extension("wma"); },
                _ => {}
            }
        }
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| DecoderError::DecoderError(format!("Format probe failed: {}", e)))?;

        let format = probed.format;

        let track = match format.default_track() {
            Some(t) => t,
            None => return Err(DecoderError::DecoderError("No audio track found".into())),
        };

        let track_id = track.id;

        let decoder_opts = DecoderOptions::default();
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .map_err(|e| DecoderError::DecoderError(format!("Codec creation failed: {}", e)))?;

        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);

        // 计算时长
        let duration_secs = if let Some(n_frames) = track.codec_params.n_frames {
            n_frames as f64 / sample_rate as f64
        } else {
            0.0
        };

        Ok(Self {
            sample_rate,
            channels,
            duration_secs,
            format,
            track: track_id,
            decoder,
            current_sample: 0,
            sample_buf: None,
            consumed_samples: 0,
        })
    }
}

impl AudioDecoder for SymphoniaDecoder {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn duration_secs(&self) -> f64 {
        self.duration_secs
    }

    fn decode_next(&mut self) -> Result<Vec<f32>, DecoderError> {
        let track_id = self.track;

        // 如果有已缓冲的样本，先消耗它们
        if let Some(ref mut sample_buf) = self.sample_buf {
            let remaining = sample_buf.len() - self.consumed_samples;
            if remaining > 0 {
                let samples: Vec<f32> = sample_buf.samples()
                    [self.consumed_samples..self.consumed_samples + remaining]
                    .iter()
                    .map(|&s| s)
                    .collect();
                self.consumed_samples += samples.len();
                self.current_sample += samples.len() as u64;
                return Ok(samples);
            }
        }
        self.sample_buf = None;
        self.consumed_samples = 0;

        // 解码下一包
        loop {
            let packet = match self.format.next_packet() {
                Ok(p) => p,
                Err(symphonia::core::errors::Error::IoError(_)) => {
                    // EOF 或流结束
                    return Ok(Vec::new());
                }
                Err(e) => {
                    return Err(DecoderError::DecoderError(format!("Read packet error: {}", e)));
                }
            };

            if packet.track_id() != track_id {
                continue; // 跳过非音频包
            }

            // 解码
            let decoded = match self.decoder.decode(&packet) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Decode error: {}", e);
                    continue; // 尝试下一包
                }
            };

            // 转换到 f32 样本
            let spec = *decoded.spec();
            let duration = decoded.capacity() as u64;
            let mut sample_buf = SampleBuffer::new(duration, spec);
            sample_buf.copy_interleaved_ref(decoded);

            let samples: Vec<f32> = sample_buf.samples().iter().map(|&s| s).collect();

            if samples.is_empty() {
                continue; // 尝试下一包
            }

            self.sample_buf = Some(sample_buf);
            self.consumed_samples = 0;

            let result = samples.clone();
            self.consumed_samples = result.len();
            self.current_sample += result.len() as u64;
            return Ok(result);
        }
    }

    fn seek(&mut self, sample: u64) -> Result<(), DecoderError> {
        // 简单的 seeking 实现 - 跳到指定样本位置
        self.current_sample = sample;
        self.sample_buf = None;
        self.consumed_samples = 0;
        // Symphonia 的 FormatReader 不直接支持 seek，我们返回错误让调用者处理
        Err(DecoderError::SeekNotSupported)
    }
}

