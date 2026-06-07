//! Θƒ│Σ╣ÉµûçΣ╗╢σñ╣σ»╝σàÑµ¿íσ¥ù
//! Σ╜┐τö¿σÄƒτöƒµûçΣ╗╢σ»╣Φ»¥µíåΘÇëµï⌐µûçΣ╗╢σñ╣∩╝îµë½µÅÅΘƒ│ΘóæµûçΣ╗╢σ╣╢σ»╝σàÑσê░µò░µì«σ║ô

use audio::metadata::TrackMetadata;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::path::Path;
use walkdir::WalkDir;

/// µö»µîüτÜäΘƒ│ΘóæµûçΣ╗╢µë⌐σ▒òσÉì
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "m4a", "aac"];

/// µúÇµƒÑµûçΣ╗╢µÿ»σÉªΣ╕║µö»µîüτÜäΘƒ│ΘóæµûçΣ╗╢
fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Σ╗ÄµûçΣ╗╢Σ╕¡µÅÉσÅûµ¡îµ¢▓Σ┐íµü»∩╝êτö¿Σ║Äσ»╝σàÑ∩╝ë
/// Φ┐öσ¢₧σàâτ╗ä∩╝Ü(title, artist, album, file_path, duration_secs, cover_url)
/// 从文件名解析歌曲信息
/// 支持格式:
///   "title - artist[album].ext" → (title, artist, album)
///   "title - artist.ext" → (title, artist, None)
///   "title.ext" → (title, None, None)
fn parse_filename(path: &Path) -> (Option<String>, Option<String>, Option<String>) {
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s.trim(),
        None => return (None, None, None),
    };

    if stem.is_empty() {
        return (None, None, None);
    }

    let (main_part, album) = if let Some(bracket_start) = stem.rfind('[') {
        if let Some(bracket_end) = stem.rfind(']') {
            if bracket_end > bracket_start {
                let album_name = stem[bracket_start + 1..bracket_end].trim();
                let main = stem[..bracket_start].trim();
                (main, Some(album_name.to_string()))
            } else {
                (stem, None)
            }
        } else {
            (stem, None)
        }
    } else {
        (stem, None)
    };

    if let Some(dash_pos) = main_part.find(" - ") {
        let title = main_part[..dash_pos].trim();
        let artist = main_part[dash_pos + 3..].trim();
        (
            if title.is_empty() { None } else { Some(title.to_string()) },
            if artist.is_empty() { None } else { Some(artist.to_string()) },
            album,
        )
    } else if !main_part.is_empty() {
        (Some(main_part.to_string()), None, album)
    } else {
        (None, None, album)
    }
}

pub fn extract_song_metadata(path: &Path) -> Option<(String, String, String, String, f64, Option<String>)> {
    let file_path_str = path.to_string_lossy().to_string();

    // 先从文件名解析
    let (path_title, path_artist, path_album) = parse_filename(path);

    // 尝试从文件元数据读取
    let meta = TrackMetadata::open(path).ok();

    // 标题：优先路径解析，其次元数据，最后默认值
    let title = path_title
        .or_else(|| meta.as_ref().and_then(|m| m.title.clone()))
        .unwrap_or_else(|| "未知标题".to_string());

    // 艺术家：优先路径解析，其次元数据，最后默认值
    let artist = path_artist
        .or_else(|| meta.as_ref().and_then(|m| m.artist.clone()))
        .unwrap_or_else(|| "未知艺术家".to_string());

    // 专辑：优先路径解析，其次元数据，最后默认值
    let album = path_album
        .or_else(|| meta.as_ref().and_then(|m| m.album.clone()))
        .unwrap_or_else(|| "未知专辑".to_string());

    // 时长：从元数据获取
    let duration_secs = meta.as_ref().map(|m| m.duration_secs).unwrap_or(0.0);

    // 封面：从元数据获取
    let cover_url = meta.as_ref().and_then(|m| {
        if let (Some(data), Some(mime)) = (&m.cover_data, &m.cover_mime) {
            let base64 = STANDARD.encode(data);
            Some(format!("data:{};base64,{}", mime, base64))
        } else {
            None
        }
    });

    Some((title, artist, album, file_path_str, duration_secs, cover_url))
}

/// µë½µÅÅΘƒ│Σ╣ÉµûçΣ╗╢σñ╣
/// σ£¿σÉÄσÅ░τ║┐τ¿ïΣ╕¡µëºΦíîµûçΣ╗╢µë½µÅÅ∩╝îΦ┐öσ¢₧Φ╖»σ╛äσêùΦí¿
#[allow(dead_code)]
pub async fn scan_music_folder(folder_path: std::path::PathBuf) -> Result<Vec<String>, String> {
    // σ£¿σÉÄσÅ░τ║┐τ¿ïΣ╕¡µëºΦíîµûçΣ╗╢µë½µÅÅ
    tokio::task::spawn_blocking(move || {
        let mut paths: Vec<String> = Vec::new();

        for entry in WalkDir::new(&folder_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if !path.is_file() || !is_audio_file(path) {
                continue;
            }

            paths.push(path.to_string_lossy().to_string());
        }

        paths
    })
    .await
    .map_err(|e| format!("Σ╗╗σèíµëºΦíîσñ▒Φ┤Ñ: {}", e))
}

/// µë½µÅÅσ╣╢σ»╝σàÑΘƒ│Σ╣ÉµûçΣ╗╢σñ╣
/// σ£¿σÉÄσÅ░τ║┐τ¿ïΣ╕¡µë½µÅÅµûçΣ╗╢∩╝îΦ┐öσ¢₧Φ╖»σ╛äσêùΦí¿∩╝îτä╢σÉÄσ£¿Σ╕╗τ║┐τ¿ïΣ╕èµÅÆσàÑµò░µì«σ║ô
pub async fn import_music_folder(db: &db::Database) -> Result<ImportResult, String> {
    use rfd::AsyncFileDialog;
    use chrono::Utc;

    // µëôσ╝ÇσÄƒτöƒµûçΣ╗╢σñ╣ΘÇëµï⌐σ»╣Φ»¥µíå
    let folder = AsyncFileDialog::new()
        .set_title("ΘÇëµï⌐Θƒ│Σ╣ÉµûçΣ╗╢σñ╣")
        .pick_folder()
        .await
        .ok_or_else(|| "µ£¬ΘÇëµï⌐µûçΣ╗╢σñ╣".to_string())?;

    let folder_path = folder.path().to_path_buf();

    // σ£¿σÉÄσÅ░τ║┐τ¿ïΣ╕¡µëºΦíîµûçΣ╗╢µë½µÅÅ
    let file_paths = tokio::task::spawn_blocking(move || {
        let mut paths: Vec<String> = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        for entry in WalkDir::new(&folder_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if !path.is_file() || !is_audio_file(path) {
                continue;
            }

            match extract_song_metadata(path) {
                Some((_, _, _, file_path, _, _)) => {
                    paths.push(file_path);
                }
                None => {
                    errors.push(format!("µùáµ│òΦ»╗σÅûµûçΣ╗╢σàâµò░µì«: {}", path.display()));
                }
            }
        }

        (paths, errors)
    })
    .await
    .map_err(|e| format!("Σ╗╗σèíµëºΦíîσñ▒Φ┤Ñ: {}", e))?;

    let (paths, scan_errors) = file_paths;

    let mut import_result = ImportResult {
        total_found: paths.len(),
        total_imported: 0,
        total_skipped: 0,
        errors: scan_errors,
    };

    // σ£¿Σ╕╗τ║┐τ¿ïΣ╕èµÅÆσàÑµò░µì«σ║ô
    let updated_covers = 0;
    for file_path in &paths {
        let path = std::path::Path::new(&file_path);
        let meta = extract_song_metadata(path);

        match db.get_song_by_path(&file_path) {
            Ok(Some(existing)) => {
                // 更新已存在歌曲：如果封面缺失、时长为0、或时长异常（>3600秒，可能是毫秒误存），则更新
                let need_cover = existing.cover_url.is_none();
                let need_duration = existing.duration_secs <= 0.0 || existing.duration_secs > 3600.0;
                
                let cover_url_opt = if need_cover {
                    if let Some((_, _, _, _, _, Some(ref cover_url))) = meta {
                        Some(cover_url.as_str())
                    } else { None }
                } else { None };
                
                let duration_opt = if need_duration {
                    if let Some((_, _, _, _, duration_secs, _)) = meta {
                        if duration_secs > 0.0 { Some(duration_secs) } else { None }
                    } else { None }
                } else { None };
                
                if need_cover || need_duration {
                    let _ = db.update_song_cover(&file_path, cover_url_opt, duration_opt);
                    println!("[Import] Updated existing: {} (cover: {}, dur: {})", 
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        need_cover,
                        need_duration
                    );
                } else {
                    println!("[Import] Skipped (up-to-date): {}", path.file_name().unwrap_or_default().to_string_lossy());
                }
                import_result.total_skipped += 1;
            }
            Ok(None) => {
                 if let Some((title, artist, album, _, duration_secs, cover_url)) = meta {
                     let song = db::Song {
                         id: None,
                         title,
                         artist,
                         album,
                         file_path: file_path.clone(),
                         duration_secs,
                         quality: "unknown".to_string(),
                         cover_url,
                         cdn_url: None,
                         date_added: Utc::now(),
                     };
                     match db.add_song(&song) {
                         Ok(_) => {
                             import_result.total_imported += 1;
                             println!("[Import]σ»╝σàÑ: {}", path.file_name().unwrap_or_default().to_string_lossy());
                         },
                         Err(e) => import_result.errors.push(format!("µ╖╗σèáσê░µò░µì«σ║ôσñ▒Φ┤Ñ: {}", e)),
                     }
                 }
            }
            Err(e) => {
                import_result.errors.push(format!("µƒÑΦ»óµò░µì«σ║ôσñ▒Φ┤Ñ: {}", e));
            }
        }
    }

    if updated_covers > 0 {
        println!("[Import]µ¢┤µû░ {} Θªûµ¡îµ¢▓σ░üΘ¥ó", updated_covers);
    }

    Ok(import_result)
}

/// σ»╝σàÑτ╗ôµ₧£
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub total_found: usize,
    pub total_imported: usize,
    pub total_skipped: usize,
    pub errors: Vec<String>,
}
