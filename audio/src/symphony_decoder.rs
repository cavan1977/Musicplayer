//! 音频解码器 (Symphonia 无损解码)
use std::fs::File;
use std::path::Path;

use symphonia::core::audio::AudioBufferRef;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::default::Decoder;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatReader, Packet};
use symphonia::core::formats::TrackId;
use symphonia::core::io::{MediaSource, MediaSourceStream, Hint};
use symphonia::core::sample::Sample;
use symphonia::default;

use crate::error::DecoderError;

/// 音频解码器：使用 Symphonia 解码 FLAC/WAV 等无损格式
pub struct AudioDecoder {
    sample_rate: u32,
    channels: u16,
    decoder: Box<dyn Decoder<Item = AudioBufferRef<'static>>>,
    reader: FormatReader<MediaSourceStream>,
    track_id: TrackId,
    duration_secs: f64,
    current_sample: u64,
    leftover: Vec<f32>,
}

impl AudioDecoder {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DecoderError> {
        // 打开文件并包装为 MediaSourceStream
        let file = File::open(path).map_err(|e| DecoderError::IoError(e.to_string()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // 探测格式
        let format = {
            let mut probe = default::get_probe();
            probe.format(&Hint::default(), mss, &Default::default(), &Default::default())
                .map_err(|e| DecoderError::FormatError(e.to_string()))?
        };

        // 获取默认音频轨道
        let track = format
            .default_track()
            .ok_or(DecoderError::NoAudioTrack)?;

        let codec_params = &track.codec_params;

        // 提取采样率和声道数
        let sample_rate = codec_params
            .sample_rate
            .ok_or(DecoderError::MissingSampleRate)?;
        let channels = codec_params
            .channels
            .ok_or(DecoderError::MissingChannels)?;

        // 计算时长（基于帧数）
        let n_frames = codec_params.n_frames.unwrap_or(0);
        let duration_secs = n_frames as f64 / sample_rate as f64;

        // 创建解码器
        let decoder = default::get_codecs()
            .make(codec_params, &DecoderOptions::default())
            .map_err(|e| DecoderError::DecoderError(e.to_string()))?;

        Ok(Self {
            sample_rate,
            channels,
            decoder,
            reader: format,
            track_id: track.id,
            duration_secs,
            current_sample: 0,
            leftover: Vec::new(),
        })
    }

    /// 解码下一批 PCM 样本 (f32, 交错)
    pub fn decode_next(&mut self) -> Result<Vec<f32>, DecoderError> {
        let mut out = Vec::new();

        loop {
            // 先使用 leftover
            if !self.leftover.is_empty() {
                out.append(&mut self.leftover);
                if out.len() >= 1024 * self.channels as usize {
                    let excess = out.split_off(1024 * self.channels as usize);
                    self.leftover = excess;
                    return Ok(out);
                }
                self.leftover.clear();
            }

            // 读取下一个 packet
            let packet = match self.reader.next_packet() {
                Ok(pkt) => pkt,
                Err(SymphoniaError::IoError(_)) => return Ok(out), // EOF
                Err(e) => return Err(DecoderError::Symphonia(e.to_string())),
            };

            // 只处理目标轨道的 packet
            if packet.track_id() != self.track_id {
                continue;
            }

            // 解码为 AudioBufferRef
            let samples: Vec<f32> = match self.decoder.decode(&packet) {
                Ok(buf) => match buf {
                    AudioBufferRef::F32(buf) => buf.iter().copied().collect(),
                    AudioBufferRef::F64(buf) => buf.iter().map(|&s| s as f32).collect(),
                    AudioBufferRef::U8(buf) => buf.iter().map(|&s| s.as_f32()).collect(),
                    AudioBufferRef::U16(buf) => buf.iter().map(|&s| s.as_f32()).collect(),
                    AudioBufferRef::U24(buf) => buf.iter().map(|&s| s.as_f32()).collect(),
                    AudioBufferRef::U32(buf) => buf.iter().map(|&s| s.as_f32()).collect(),
                    _ => return Err(DecoderError::UnsupportedSampleFormat),
                },
                Err(e) => return Err(DecoderError::Symphonia(e.to_string())),
            };

            self.current_sample += samples.len() as u64 / self.channels as u64;
            out.extend(samples);

            if out.len() >= 1024 * self.channels as usize {
                let excess = out.split_off(1024 * self.channels as usize);
                self.leftover = excess;
                return Ok(out);
            }
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// 获取总时长 (秒)
    pub fn duration_secs(&self) -> f64 {
        self.duration_secs
    }

    pub fn current_sample(&self) -> u64 {
        self.current_sample
    }
}

// Helper to get the track_id from the reader (we store it separately)
// We'll modify struct to store track_id as well. Update struct:
// Add track_id: u32 (or TrackId). But we can just compare packet.track_id() to the track.id of the default track. Since we can't store reference, we store the numeric id.
// Let's add field: track_id: u64 (or u32).
// But we haven't stored it. Let's adjust struct to include track_id.
// But easier: we can skip track filtering entirely; just accept all packets from the only track? However if multiple tracks, we should filter. Let's store track ID.
// Add `track_id: u32` in struct. Set in open: `let track_id = track.id;` but track.id is of type TrackId which likely is a u32 newtype. Use `track.id.0` or `track.id`? Actually TrackId might be a tuple struct with u32. We'll check. For now, we can store as u32.
// We'll need to add field.
