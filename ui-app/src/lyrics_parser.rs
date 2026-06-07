// LRC Lyrics Parser Module
// 支持: LRC 外部文件 + 内嵌歌词(fallback to TrackMetadata)

use std::fs;
use std::path::Path;

/// Parsed lyric line with timestamp
#[derive(Debug, Clone)]
pub struct LyricLine {
    pub time: f64, // seconds
    pub text: String,
}

/// Parsed lyrics container
#[derive(Debug, Clone)]
pub struct Lyrics {
    pub lines: Vec<LyricLine>,
}

impl Lyrics {
    pub fn from_text(text: String) -> Self {
        Self {
            lines: vec![LyricLine { time: 0.0, text }],
        }
    }
}

/// Parse LRC format lyrics
pub fn parse_lrc(content: &str) -> Lyrics {
    let mut lines = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if let Some(end) = line.find(']') {
            let time_str = &line[1..end];
            if let Ok(seconds) = parse_time(time_str) {
                let text = line[end+1..].trim().to_string();
                if !text.is_empty() {
                    lines.push(LyricLine { time: seconds, text });
                }
            }
        }
    }
    lines.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    Lyrics { lines }
}

fn parse_time(s: &str) -> Result<f64, ()> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 { return Err(()); }
    Ok(parts[0].parse::<f64>().map_err(|_| ())? * 60.0 + parts[1].parse::<f64>().map_err(|_| ())?)
}

/// Extract embedded lyrics via audio crate's TrackMetadata (lofty-based)
fn extract_embedded_lyrics(audio_path: &Path) -> Option<String> {
    match audio::metadata::TrackMetadata::open(audio_path) {
        Ok(meta) => {
            if let Some(ref lyrics) = meta.embedded_lyrics {
                println!("[Lyrics] Successfully extracted embedded lyrics (len={})", lyrics.len());
                Some(lyrics.clone())
            } else {
                println!("[Lyrics] TrackMetadata opened but no embedded_lyrics field");
                None
            }
        }
        Err(e) => {
            println!("[Lyrics] TrackMetadata::open failed: {}", e);
            None
        }
    }
}

/// Load lyrics: try .lrc file first, then embedded lyrics
pub fn load_lyrics_for_song<P: AsRef<Path>>(audio_path: P) -> Option<Lyrics> {
    let path = audio_path.as_ref();
    println!("[Lyrics] Loading for: {:?}", path);

    // 1. 尝试外部 .lrc 文件 (两种命名: stem.lrc 和 filename.lrc)
    if let Some(stem) = path.file_stem() {
        let lrc_path1 = path.with_file_name(format!("{}.lrc", stem.to_string_lossy()));
        println!("[Lyrics] Trying stem.lrc: {:?} exists={}", lrc_path1, lrc_path1.exists());
        if lrc_path1.exists() {
            if let Ok(content) = fs::read_to_string(&lrc_path1) {
                let lyrics = parse_lrc(&content);
                if !lyrics.lines.is_empty() { 
                    println!("[Lyrics] Loaded from stem.lrc ({} lines)", lyrics.lines.len());
                    return Some(lyrics); 
                }
            }
        }
    }
    if let Some(file_name) = path.file_name() {
        let lrc_name = format!("{}.lrc", file_name.to_string_lossy());
        let lrc_path2 = path.with_file_name(lrc_name);
        println!("[Lyrics] Trying filename.lrc: {:?} exists={}", lrc_path2, lrc_path2.exists());
        if lrc_path2.exists() {
            if let Ok(content) = fs::read_to_string(&lrc_path2) {
                let lyrics = parse_lrc(&content);
                if !lyrics.lines.is_empty() { 
                    println!("[Lyrics] Loaded from filename.lrc ({} lines)", lyrics.lines.len());
                    return Some(lyrics); 
                }
            }
        }
    }

    // 2. 回退到内嵌歌词
    println!("[Lyrics] Trying embedded...");
    if let Some(embedded) = extract_embedded_lyrics(path) {
        println!("[Lyrics] Embedded lyrics found, len={}", embedded.len());
        if embedded.contains('[') && embedded.contains(']') && embedded.contains(':') {
            let lyrics = parse_lrc(&embedded);
            if !lyrics.lines.is_empty() { 
                println!("[Lyrics] Parsed embedded LRC ({} lines)", lyrics.lines.len());
                return Some(lyrics); 
            }
        }
        let lyrics = Lyrics::from_text(embedded);
        println!("[Lyrics] Using as plain text (1 line)");
        return Some(lyrics);
    }

    println!("[Lyrics] No lyrics found");
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_lrc() {
        let lrc = r#"[00:12.34]Hello world
[00:15.00]This is a test
[00:10.00]Earlier line
"#;
        let lyrics = parse_lrc(lrc);
        assert_eq!(lyrics.lines.len(), 3);
        assert_eq!(lyrics.lines[0].time, 10.0);
        assert_eq!(lyrics.lines[1].time, 12.34);
        assert_eq!(lyrics.lines[2].time, 15.0);
    }
}