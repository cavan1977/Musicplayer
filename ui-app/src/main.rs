#![allow(non_snake_case)]
#![windows_subsystem = "windows"]

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder, LogicalSize};
use dioxus_desktop::tao::platform::windows::WindowBuilderExtWindows;
use audio::Player;
use audio::PlaybackMode;
use db::Database;
use tokio::time::{sleep, Duration};

mod theme;
mod aqua;
mod sonic;
mod vintage;
mod lyrics_parser;
mod import;

use theme::{UiStyle, ThemeManager};
use lyrics_parser::Lyrics;
use import::import_music_folder;

fn main() {
    let virtual_dom = VirtualDom::new(app);
    let config = Config::default()
        .with_window(
            WindowBuilder::new()
                .with_title("HiFi 水琉璃音乐")
                .with_decorations(false)
                .with_undecorated_shadow(false)
                .with_transparent(true)
                .with_inner_size(LogicalSize::new(1000.0, 760.0))
                .with_min_inner_size(LogicalSize::new(1000.0, 760.0))
                .with_max_inner_size(LogicalSize::new(1000.0, 760.0))
                .with_resizable(false),
        );
    dioxus_desktop::launch::launch_virtual_dom(virtual_dom, config);
}

// Re-export Song from db module for convenience
pub use db::Song;

fn format_time(seconds: f64) -> String {
    let mins = seconds as u64 / 60;
    let secs = seconds as u64 % 60;
    format!("{}:{:02}", mins, secs)
}

#[component]
fn app() -> Element {
    // Theme state
    let theme = use_signal(|| ThemeManager::new(UiStyle::AquaGlass));

    // Other state
    let player = use_signal(|| Player::new());
    let db = use_signal(|| {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let db_path = exe_dir.join("musicplayer.db");
        Database::open(&db_path).expect("Failed to open DB")
    });
    let current_song = use_signal(|| None::<Song>);
    let is_playing = use_signal(|| false);
    let progress = use_signal(|| 0.0);
    let progress_secs = use_signal(|| 0.0);
    let volume = use_signal(|| 0.8);
    let mut lyrics = use_signal(|| None::<Lyrics>);
    let spectrum = use_signal(|| [0.0f32; 64]);
    let refresh_songs = use_signal(|| 0u32);
    let playback_mode = use_signal(|| PlaybackMode::Sequential);
    let is_seeking = use_signal(|| false);
    let output_devices = use_signal(|| Player::list_output_devices());

    {
        let db = db.clone();
        let player = player.clone();
        let mut current_song = current_song.clone();
        let mut is_playing = is_playing.clone();
        let mut refresh_songs = refresh_songs.clone();
        let mut lyrics = lyrics.clone();
        use_future(move || async move {
            let args: Vec<String> = std::env::args().collect();
            if args.len() < 2 {
                return;
            }
            let file_path = args[1].clone();
            let path = std::path::Path::new(&file_path);
            if !path.exists() || !path.is_file() {
                return;
            }

            let metadata = match crate::import::extract_song_metadata(path) {
                Some(m) => m,
                None => return,
            };

            let (title, artist, album, _, duration, cover_url) = metadata;
            use chrono::Utc;
            let song = db::Song {
                id: None, title, artist, album,
                file_path: file_path.clone(),
                duration_secs: duration,
                quality: "lossless".into(),
                cover_url, cdn_url: None,
                date_added: Utc::now(),
            };

            let db_guard = db.read();
            let db_ref = &*db_guard;
            let existing = db_ref.get_song_by_path(&file_path).unwrap_or(None);
            let song = if let Some(s) = existing {
                s
            } else {
                if db_ref.add_song(&song).is_err() {
                    drop(db_guard);
                    return;
                }
                db_ref.get_song_by_path(&file_path).unwrap_or(None).unwrap_or(song)
            };
            drop(db_guard);

            refresh_songs.set(refresh_songs() + 1);

            match player().play(&song.file_path) {
                Ok(_) => {
                    current_song.set(Some(song.clone()));
                    is_playing.set(true);
                    if let Ok(all_songs) = db.read().list_songs() {
                        if let Some(idx) = all_songs.iter().position(|s| s.id == song.id) {
                            let paths: Vec<std::path::PathBuf> = all_songs.iter()
                                .map(|s| std::path::PathBuf::from(&s.file_path))
                                .collect();
                            player().set_playlist(paths, idx);
                        }
                    }
                    let lrc = crate::lyrics_parser::load_lyrics_for_song(&song.file_path);
                    lyrics.set(lrc);
                }
                Err(_) => {}
            }
        });
    }

    let on_import = Callback::new(move |_| {
        let db_clone = db.clone();
        let mut refresh_songs = refresh_songs.clone();
        spawn(async move {
            let db_guard = db_clone.read();
            let db_ref = &*db_guard;
            match import_music_folder(db_ref).await {
                Ok(result) => {
                    println!("[Import]完成: 找到{}首, 导入{}, 跳过{}",
                        result.total_found, result.total_imported, result.total_skipped);
                    if !result.errors.is_empty() {
                        println!("[Import]错误: {:?}", result.errors);
                    }
                    refresh_songs.set(refresh_songs() + 1);
                }
                Err(e) => {
                    println!("[Import]失败: {}", e);
                }
            }
        });
    });

    // Progress polling
    {
        let player = player.clone();
        let mut progress = progress.clone();
        let mut progress_secs = progress_secs.clone();
        let is_seeking = is_seeking.clone();
        use_future(move || async move {
            loop {
                sleep(Duration::from_millis(200)).await;
                if *is_seeking.read() {
                    continue;
                }
                let dur = player().duration_secs();
                if dur > 0.0 {
                    let secs = player().progress_secs();
                    let pct = (secs / dur) * 100.0;
                    progress.set(pct);
                    progress_secs.set(secs);
                } else {
                    progress_secs.set(0.0);
                }
            }
        });
    }

     // Spectrum polling
     {
         let player = player.clone();
         let mut spectrum_sig = spectrum.clone();
         use_future(move || async move {
             loop {
                 sleep(Duration::from_millis(33)).await;
                 let bands = player().get_spectrum_bands();
                 spectrum_sig.set(bands);
             }
         });
     }

     // Auto-advance queue when track ends (and sync UI state)
     {
         let player = player.clone();
         let db = db.clone();
         let mut current_song = current_song.clone();
         let mut is_playing = is_playing.clone();
         let mut lyrics = lyrics.clone();
         use_future(move || async move {
             loop {
                 sleep(Duration::from_millis(500)).await;
                 if player().sink_empty() {
                     player().advance_queue();
                     if let Some(path) = player().current_file_path() {
                         let path_str = path.to_string_lossy().to_string();
                         let db_guard = db.read();
                         if let Ok(Some(song)) = db_guard.get_song_by_path(&path_str) {
                             drop(db_guard);
                             current_song.set(Some(song.clone()));
                             is_playing.set(true);
                             let lrc = crate::lyrics_parser::load_lyrics_for_song(&song.file_path);
                             lyrics.set(lrc);
                         } else {
                             drop(db_guard);
                         }
                     } else {
                         current_song.set(None);
                         is_playing.set(false);
                     }
                 }
             }
         });
     }

     // File drag-and-drop support
     {
         let db = db.clone();
         let player = player.clone();
         let mut refresh_songs = refresh_songs.clone();
         let mut is_playing = is_playing.clone();
         use_future(move || async move {
             // Inject native drop listener via JS
                 let _ = dioxus::document::eval(
                 r#"
                 document.addEventListener('drop', function(e) {
                     e.preventDefault();
                     const files = Array.from(e.dataTransfer.files);
                     window.__droppedFiles = files.map(f => f.path || f.name).join('\n');
                 });
                 document.addEventListener('dragover', function(e) {
                     e.preventDefault();
                 });
                 "#
             ).await;
             // Poll for dropped files
             loop {
                 sleep(Duration::from_millis(800)).await;
                 let result = dioxus::document::eval(
                     r#"(() => { const p = window.__droppedFiles; window.__droppedFiles = undefined; return p; })()"#
                 ).await;
                 if let Ok(val) = result {
                     if let Some(paths_str) = val.as_str() {
                         if paths_str.is_empty() || paths_str == "undefined" { continue; }
                         let paths: Vec<String> = paths_str.split('\n').map(|s: &str| s.trim().to_string()).filter(|s: &String| !s.is_empty()).collect();
                         if paths.is_empty() { continue; }
                         println!("[Drop] 拖入 {} 个文件", paths.len());
                         let db_guard = db.read();
                         let db_ref = &*db_guard;
                         for fp in &paths {
                             let path = std::path::Path::new(fp);
                             if !path.is_file() { continue; }
                             if db_ref.get_song_by_path(fp).unwrap_or(None).is_some() {
                                 continue;
                             }
                             if let Some((title, artist, album, _, duration, cover_url)) =
                                 crate::import::extract_song_metadata(path)
                             {
                                 use chrono::Utc;
                                 let song = db::Song {
                                     id: None, title, artist, album,
                                     file_path: fp.clone(),
                                     duration_secs: duration,
                                     quality: "lossless".into(),
                                     cover_url, cdn_url: None,
                                     date_added: Utc::now(),
                                 };
                                 if let Err(e) = db_ref.add_song(&song) {
                                     eprintln!("[Drop] 导入失败 {}: {}", fp, e);
                                 }
                             }
                         }
                         drop(db_guard);
                         refresh_songs.set(refresh_songs() + 1);
                         if let Some(first) = paths.first() {
                             let _ = player().play(first);
                         }
                     }
                 }
             }
         });
     }

    // Get current theme style
    let current_style = theme.read().current;
    let sheet = theme.read().style_sheet.clone();

    // Render the appropriate theme based on current style
    let scrollbar_css = "::-webkit-scrollbar { display: none; }";
    let html_body_css = "
        html, body {
            margin: 0;
            padding: 0;
            border: none;
            overflow: hidden;
            background: transparent;
        }
    ";
     rsx! {
         style { r#type: "text/css", "{sheet.to_css_vars()}" }
         style { "{sheet.animations}" }
         style { "{scrollbar_css}" }
         style { "{html_body_css}" }
        div {
            style: "
                width: 100vw;
                height: 100vh;
                background: var(--bg-start);
                color: var(--text-primary);
                font-family: var(--font-family);
                display: flex;
                flex-direction: column;
                overflow: hidden;
            ",
              match current_style {
                   UiStyle::AquaGlass => {
                       rsx! {
                          aqua::App {
                              theme: theme.clone(),
                              player: player.clone(),
                              db: db.clone(),
                              current_song: current_song.clone(),
                              is_playing: is_playing.clone(),
                              progress: progress.clone(),
                              progress_secs: progress_secs.clone(),
                              volume: volume.clone(),
                              lyrics: lyrics.clone(),
                              on_import: on_import.clone(),
                              spectrum: spectrum.clone(),
                              refresh_songs: refresh_songs.clone(),
                              playback_mode: playback_mode.clone(),
                              is_seeking: is_seeking.clone(),
                              output_devices: output_devices.clone(),
                          }
                       }
                   }
                   UiStyle::SonicFlux => {
                       rsx! {
                          sonic::App {
                              theme: theme.clone(),
                              player: player.clone(),
                              db: db.clone(),
                              current_song: current_song.clone(),
                              is_playing: is_playing.clone(),
                              progress: progress.clone(),
                              progress_secs: progress_secs.clone(),
                              volume: volume.clone(),
                              lyrics: lyrics.clone(),
                              on_import: on_import.clone(),
                              spectrum: spectrum.clone(),
                              refresh_songs: refresh_songs.clone(),
                              playback_mode: playback_mode.clone(),
                              is_seeking: is_seeking.clone(),
                              output_devices: output_devices.clone(),
                          }
                       }
                   }
                   UiStyle::VintagePro => {
                       rsx! {
                    vintage::App {
                        theme: theme.clone(),
                        player: player.clone(),
                        db: db.clone(),
                        current_song: current_song.clone(),
                        is_playing: is_playing.clone(),
                        progress: progress.clone(),
                        progress_secs: progress_secs.clone(),
                        volume: volume.clone(),
                        lyrics: lyrics.clone(),
                        on_import: on_import.clone(),
                        spectrum: spectrum.clone(),
                        refresh_songs: refresh_songs.clone(),
                        playback_mode: playback_mode.clone(),
                        is_seeking: is_seeking.clone(),
                        output_devices: output_devices.clone(),
                    }
                       }
                   }
              }
        }
    }
}
