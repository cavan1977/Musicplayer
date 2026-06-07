//! 音频文件原始元数据（基于 audiotags + lofty）
use std::path::Path;

use audiotags::Tag;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::prelude::ItemKey;

use crate::error::DecoderError;

#[derive(Debug, Clone)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u16,
    // 来自 lofty 额外字段
    pub cover_mime: Option<String>,
    pub cover_data: Option<Vec<u8>>,
    pub embedded_lyrics: Option<String>,
}

impl TrackMetadata {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DecoderError> {
        let p: &Path = path.as_ref();
        let tag = Tag::new()
            .read_from_path(p)
            .map_err(|e| DecoderError::DecoderError(e.to_string()))?;

        let title = tag.title().map(|s| s.to_string());
        let artist = tag.artist().map(|s| s.to_string());
        let album = tag.album_title().map(|s| s.to_string());

        // ---------- 用 lofty 获取封面/歌词/时长 ----------
        // 注意：audiotags 的 duration() 对 MP3 (ID3v2) 返回的是毫秒而非秒，
        // 这是 audiotags 的 bug（TLEN 帧单位是毫秒但未做转换）。
        // 改用 lofty 获取时长，lofty 的 TaggedFileExt::duration() 返回正确的秒数。
        let (cover_mime, cover_data, embedded_lyrics, duration_secs) =
            Self::extended_meta_lofty(p, tag.duration());

        Ok(Self {
            title,
            artist,
            album,
            duration_secs,
            sample_rate: 0,
            channels: 0,
            cover_mime,
            cover_data,
            embedded_lyrics,
        })
    }

    /// 用 lofty 补充封面、歌词、时长。
    /// `audiotags_duration`: audiotags 返回的时长（对 MP3 可能有误），仅作后备。
    fn extended_meta_lofty(
        file_path: &Path,
        audiotags_duration: Option<f64>,
    ) -> (Option<String>, Option<Vec<u8>>, Option<String>, f64) {
        let mut cover_mime = None;
        let mut cover_data: Option<Vec<u8>> = None;
        let mut lyrics: Option<String> = None;

        let tagged_file = match lofty::read_from_path(file_path) {
            Ok(f) => f,
            Err(_) => {
                // lofty 读取失败，回退到 audiotags 的时长
                return (cover_mime, cover_data, lyrics, audiotags_duration.unwrap_or(0.0));
            }
        };

        // 优先使用 lofty 的时长（准确）
        let duration_secs = tagged_file.properties().duration().as_secs_f64();
        // 如果 lofty 时长为 0，回退到 audiotags（但需修正 MP3 毫秒问题）
        let duration_secs = if duration_secs > 0.0 {
            duration_secs
        } else if let Some(d) = audiotags_duration {
            // 启发式判断：如果时长 > 3600（1小时），很可能是毫秒误当秒
            if d > 3600.0 {
                d / 1000.0
            } else {
                d
            }
        } else {
            0.0
        };

        let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

        if let Some(tag) = tag {
            // ----- 封面图片 -----
            for pic in tag.pictures() {
                if let Some(mime) = pic.mime_type() {
                    cover_mime = Some(mime.as_str().to_string());
                } else {
                    cover_mime = Some("image/jpeg".into());
                }
                cover_data = Some(pic.data().to_vec());
                break;
            }

            // Try Lyrics key
            if let Some(s) = tag.get_string(&ItemKey::Lyrics) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    lyrics = Some(trimmed.to_string());
                }
            }
        }

        (cover_mime, cover_data, lyrics, duration_secs)
    }
}