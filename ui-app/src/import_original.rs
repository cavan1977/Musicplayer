//! Θƒ│Σ╣ÉµûçΣ╗╢σñ╣σ»╝σàÑµ¿íσ¥ù
//! Σ╜┐τö¿σÄƒτöƒµûçΣ╗╢σ»╣Φ»¥µíåΘÇëµï⌐µûçΣ╗╢σñ╣∩╝îµë½µÅÅΘƒ│ΘóæµûçΣ╗╢σ╣╢σ»╝σàÑσê░µò░µì«σ║ô

use audiotags::Tag;
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
pub fn extract_song_metadata(path: &Path) -> Option<(String, String, String, String, f64, Option<String>)> {
    let tag = Tag::new().read_from_path(path).ok()?;
    let file_path_str = path.to_string_lossy().to_string();

    // σ░¥Φ»òΣ╗Äµáçτ¡╛ΦÄ╖σÅûΣ┐íµü»∩╝îσñ▒Φ┤ÑσêÖΣ╜┐τö¿µûçΣ╗╢σÉì
    let title = tag
        .title()
        .map(|s: &str| s.to_string())
        .or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s: &str| s.to_string())
        })
        .unwrap_or_else(|| "µ£¬τƒÑµáçΘóÿ".to_string());

    let artist = tag
        .artist()
        .map(|s: &str| s.to_string())
        .unwrap_or_else(|| "µ£¬τƒÑΦë║µ£»σ«╢".to_string());

    let album = tag
        .album()
        .map(|a: audiotags::Album| a.title.to_string())
        .unwrap_or_else(|| "µ£¬τƒÑΣ╕ôΦ╛æ".to_string());

    // ΦÄ╖σÅûµù╢Θò┐ - audiotags 0.4 Σ╕ìτ¢┤µÄÑµÅÉΣ╛¢µù╢Θò┐∩╝îµÜéµù╢Φ«╛τ╜«Σ╕║0
    let duration_secs: f64 = 0.0;

    // µÅÉσÅûσ░üΘ¥ó
    let cover_url = if let Some(picture) = tag.album_cover() {
        // picture.data µÿ» &[u8], picture.mime_type µÿ» MimeType
        let mime = match picture.mime_type {
            audiotags::MimeType::Jpeg => "image/jpeg",
            audiotags::MimeType::Png => "image/png",
            _ => "image/jpeg",
        };
        let base64 = base64::engine::general_purpose::STANDARD.encode(picture.data);
        Some(format!("data:{};base64,{}", mime, base64))
    } else {
        None
    };

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
    let mut updated_covers = 0;
    for file_path in &paths {
        let path = std::path::Path::new(&file_path);
        let meta = extract_song_metadata(path);

        match db.get_song_by_path(&file_path) {
            Ok(Some(existing)) => {
                // σ╖▓σ¡ÿσ£¿∩╝Üµ¢┤µû░σ░üΘ¥ó∩╝êσªéµ₧£ DB Σ╕¡Σ╕║τ⌐║Σ╜åµûçΣ╗╢µ£ëσåàσ╡îσ░üΘ¥ó∩╝ë
                if existing.cover_url.is_none() {
                    if let Some((_, _, _, _, _, Some(ref cover_url))) = meta {
                        let _ = db.update_song_cover(&file_path, Some(cover_url.as_str()), None);
                        updated_covers += 1;
                        println!("[Import]µ¢┤µû░σ░üΘ¥ó: {}", path.file_name().unwrap_or_default().to_string_lossy());
                    }
                } else {
                    // σ░üΘ¥óσ¡ÿσ£¿∩╝îΣ╜åΦ┐ÿµÿ»µ¢┤µû░Σ╕ÇΣ╕ïΣ╗ÑΘÿ▓µûçΣ╗╢µ¢┤µû░
                    if let Some((_, _, _, _, _, Some(ref cover_url))) = meta {
                        let _ = db.update_song_cover(&file_path, Some(cover_url.as_str()), None);
                    }
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
