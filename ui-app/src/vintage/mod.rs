//! Vintage Pro Theme - Classical HiFi design
//! 无边框设计，使用间距、阴影和背景差异进行分隔

use dioxus::prelude::*;
use dioxus_desktop::use_window;
use db::Database;
use audio::Player;
use audio::PlaybackMode;

use crate::lyrics_parser::Lyrics;
use crate::{Song, format_time, UiStyle, ThemeManager};

fn fmt_time(seconds: f64) -> String {
    format_time(seconds)
}

// ==================== TitleBar ====================
#[derive(Props, PartialEq, Clone)]
struct TitleBarProps {
    pub theme: Signal<ThemeManager>,
    pub on_import: Callback<()>,
}

#[component]
fn TitleBar(props: TitleBarProps) -> Element {
    let mut theme = props.theme;
    let mut switch = move |style: UiStyle| {
        theme.write().switch(style);
    };
    let window = use_window();
    let window_min = window.clone();
    let window_close = window.clone();
    let on_import = props.on_import;

    rsx! {
        div {
            style: "
                height: 52px;
                background: linear-gradient(180deg, rgba(44,36,22,1) 0%, rgba(30,24,16,1) 100%);
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 0 20px;
                position: relative;
                box-shadow: 0 2px 8px rgba(0,0,0,0.3);
                user-select: none;
                -webkit-app-region: drag;
            ",
            div {
                style: "
                    position: absolute;
                    bottom: 0;
                    left: 0;
                    right: 0;
                    height: 1px;
                    background: linear-gradient(90deg, transparent 0%, var(--primary) 20%, var(--primary) 80%, transparent 100%);
                "
            },
            div {
                style: "
                    position: absolute;
                    top: 0;
                    left: 0;
                    right: 0;
                    height: 1px;
                    background: linear-gradient(90deg, transparent, rgba(255,204,122,0.15), transparent);
                "
            },
            div {
                style: "display: flex; align-items: center; gap: 12px; -webkit-app-region: no-drag;",
                div {
                    style: "
                        width: 30px;
                        height: 30px;
                        background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%);
                        border-radius: 50%;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        box-shadow: 0 2px 6px rgba(0,0,0,0.4), inset 0 1px 2px rgba(255,255,255,0.2);
                    ",
                    span { style: "font-size: 13px; color: #1a1510;", "♫" }
                }
                span {
                    style: "
                        font-size: 17px;
                        font-weight: 600;
                        color: var(--text-primary);
                        font-family: var(--font-family);
                        letter-spacing: 3px;
                    ",
                    "VINTAGE PRO"
                }
            }
            div {
                style: "display: flex; align-items: center; gap: 4px; -webkit-app-region: no-drag;",
                button {
                    onclick: move |_| switch(UiStyle::AquaGlass),
                    style: "
                        padding: 6px 14px;
                        background: rgba(212,165,116,0.06);
                        border: 1px solid rgba(212,165,116,0.2);
                        color: var(--text-secondary);
                        cursor: pointer;
                        font-size: 11px;
                        font-family: var(--font-family);
                        transition: all 0.2s;
                        border-radius: 3px;
                    ",
                    "海洋之声"
                }
                button {
                    onclick: move |_| switch(UiStyle::SonicFlux),
                    style: "
                        padding: 6px 14px;
                        background: rgba(212,165,116,0.06);
                        border: 1px solid rgba(212,165,116,0.2);
                        color: var(--text-secondary);
                        cursor: pointer;
                        font-size: 11px;
                        font-family: var(--font-family);
                        transition: all 0.2s;
                        border-radius: 3px;
                    ",
                    "赛博空间"
                }
                button {
                    onclick: move |_| switch(UiStyle::VintagePro),
                    style: "
                        padding: 6px 14px;
                        background: rgba(212,165,116,0.15);
                        border: 1px solid var(--primary);
                        color: var(--text-primary);
                        cursor: pointer;
                        font-size: 11px;
                        font-family: var(--font-family);
                        transition: all 0.2s;
                        border-radius: 3px;
                        box-shadow: 0 0 6px rgba(212,165,116,0.2);
                    ",
                    "古典HiFi"
                }
                div {
                    style: "width: 1px; height: 18px; background: rgba(212,165,116,0.2); margin: 0 4px;"
                }
                button {
                    onclick: move |_| on_import.call(()),
                    style: "
                        padding: 6px 14px;
                        border: 1px solid var(--color-accent);
                        background: rgba(212,165,116,0.08);
                        color: var(--color-accent);
                        cursor: pointer;
                        font-size: 11px;
                        font-family: var(--font-family);
                        border-radius: 3px;
                    ",
                    "导入"
                }
                button {
                    onclick: move |_| window_min.set_minimized(true),
                    style: "
                        width: 26px;
                        height: 26px;
                        border-radius: 3px;
                        border: 1px solid rgba(212,165,116,0.25);
                        background: rgba(212,165,116,0.08);
                        color: var(--text-secondary);
                        cursor: pointer;
                        font-size: 13px;
                        font-family: var(--font-family);
                    ",
                    "−"
                }
                button {
                    onclick: move |_| window_close.close(),
                    style: "
                        width: 26px;
                        height: 26px;
                        border-radius: 3px;
                        border: none;
                        background: rgba(180,60,50,0.75);
                        color: white;
                        cursor: pointer;
                        font-size: 12px;
                        font-family: var(--font-family);
                    ",
                    "✕"
                }
            }
        }
    }
}

// ==================== Sidebar ====================
#[derive(Props, PartialEq, Clone)]
struct SidebarProps {
    pub db: Signal<Database>,
    pub current_song: Signal<Option<Song>>,
    pub on_select: Callback<Song>,
    pub refresh_songs: Signal<u32>,
}

 #[component]
 fn Sidebar(props: SidebarProps) -> Element {
     let db_sig = props.db.clone();
     let mut songs: Signal<Vec<Song>> = use_signal(move || db_sig.read().list_songs().unwrap_or_default());
     let on_select = props.on_select.clone();
     let current_song = props.current_song.clone();
     let refresh = props.refresh_songs;

     use_effect(move || {
         let _ = *refresh.read();
         songs.set(db_sig.read().list_songs().unwrap_or_default());
     });

     let songs_guard = songs.read();

     rsx! {
         div {
             style: "
                 width: 280px;
                 height: 100%;
                 background: linear-gradient(180deg, rgba(44,36,22,0.95) 0%, rgba(26,21,16,0.95) 100%);
                 padding: 20px 16px;
                 display: flex;
                 flex-direction: column;
                 gap: 12px;
                 position: relative;
                 box-shadow: 2px 0 8px rgba(0,0,0,0.3);
             ",
             div {
                 style: "
                     position: absolute;
                     top: 8px;
                     right: 8px;
                     bottom: 8px;
                     left: 8px;
                     border: 1px solid rgba(212,165,116,0.12);
                     pointer-events: none;
                 "
             }
             div {
                 style: "
                     font-size: 13px;
                     font-weight: 600;
                     color: var(--primary);
                     padding-bottom: 10px;
                     border-bottom: 1px solid rgba(212,165,116,0.2);
                     font-family: var(--font-family);
                     letter-spacing: 2px;
                     text-align: center;
                 ",
                 "黑胶唱片库"
             }
             div {
                 style: "
                     flex: 1;
                     overflow-y: auto;
                     display: flex;
                     flex-direction: column;
                     gap: 4px;
                 ",
                  for song in songs_guard.iter().cloned() {
                      {
                         let song_cb = song.clone();
                         let is_current = current_song.read().as_ref().map(|s| s.file_path == song.file_path).unwrap_or(false);
                         rsx! {
                             div {
                                 key: "{song.file_path}",
                                  onclick: move |_| on_select.call(song_cb.clone()),
                                 style: format!(
                                     "padding: 10px 14px; background: {}; border-radius: 4px; cursor: pointer; transition: all 0.2s; display: flex; flex-direction: column; gap: 4px; border-left: 3px solid {};",
                                     if is_current { "rgba(212,165,116,0.12)" } else { "rgba(212,165,116,0.03)" },
                                     if is_current { "var(--primary)" } else { "transparent" }
                                 ),
                                 div {
                                     style: "display: flex; justify-content: space-between; align-items: center;",
                                     span {
                                         style: format!(
                                             "font-size: 14px; font-weight: {}; color: {}; font-family: var(--font-family); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                             if is_current { "600" } else { "400" },
                                             if is_current { "var(--text-primary)" } else { "var(--text-secondary)" }
                                         ),
                                         "{&song.title}"
                                     }
                                     span {
                                         style: "font-size: 11px; color: rgba(212,165,116,0.4); font-family: var(--font-family); flex-shrink: 0; margin-left: 8px;",
                                         "{fmt_time(song.duration_secs)}"
                                     }
                                 }
                                 span {
                                     style: "font-size: 12px; color: rgba(245,241,230,0.35); font-family: var(--font-family);",
                                     "{&song.artist}"
                                 }
                             }
                         }
                     }
                 }
             }
         }
     }
}

// ==================== MainArea ====================
#[derive(Props, PartialEq, Clone)]
struct MainAreaProps {
    pub current_song: Signal<Option<Song>>,
    pub lyrics: Signal<Option<Lyrics>>,
    pub progress_secs: Signal<f64>,
    pub is_playing: Signal<bool>,
}

#[component]
fn MainArea(props: MainAreaProps) -> Element {
    let current_song = props.current_song;
    let lyrics = props.lyrics;
    let progress_secs = props.progress_secs;
    let is_playing = props.is_playing;

    let _duration = current_song.read().as_ref().map(|s| s.duration_secs).unwrap_or(0.0);
    let _current_sec = *progress_secs.read();

    let elapsed_secs = *progress_secs.read();
    let active_text: Option<(Vec<(f64, String)>, usize)> = if let Some(lrc) = &*lyrics.read() {
        let mut lines: Vec<(f64, String)> = lrc.lines.iter()
            .filter(|line| line.time >= 0.0)
            .map(|line| (line.time, line.text.clone()))
            .collect();
        lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let active_idx = lines.iter().rposition(|&(time, _)| elapsed_secs >= time).unwrap_or(0);
        Some((lines, active_idx))
    } else {
        None
    };
    let _active_idx_opt = active_text.as_ref().map(|(_, idx)| *idx);

    let prev_active_idx: Signal<Option<usize>> = use_signal(|| None);

    {
        let progress_secs = progress_secs.clone();
        let lyrics = lyrics.clone();
        let mut prev_active_idx = prev_active_idx.clone();
        use_effect(move || {
            let elapsed = *progress_secs.read();
            let lrc_opt = lyrics.read();
            if let Some(lrc) = lrc_opt.as_ref() {
                if !lrc.lines.is_empty() {
                    let active_idx = lrc.lines.iter()
                        .rposition(|line| line.time <= elapsed)
                        .unwrap_or(0);
                    let prev = *prev_active_idx.read();
                    if prev != Some(active_idx) {
                        let js = format!(
                            "var c=document.getElementById('lyrics-container-vintage'); \
                             var l=document.getElementById('lyric-line-vintage-{}'); \
                             if(c&&l){{ \
                                 l.scrollIntoView({{behavior:'smooth', block:'center', inline:'nearest'}}); \
                             }}",
                            active_idx
                        );
                        let _ = dioxus::document::eval(&js);
                        prev_active_idx.set(Some(active_idx));
                    }
                }
            }
        });
    }

    rsx! {
        div {
            style: "
                flex: 1;
                display: flex;
                flex-direction: column;
                padding: 24px;
                gap: 20px;
                position: relative;
                overflow: hidden;
            ",
            div {
                style: "
                    position: absolute;
                    top: 10px;
                    left: 10px;
                    right: 10px;
                    bottom: 10px;
                    border: 1px solid rgba(212,165,116,0.1);
                    pointer-events: none;
                "
            }
            // Top section: Cover + Song Info
            div {
                style: "display: flex; align-items: center; gap: 24px; flex-shrink: 0;",
                div {
                    style: "
                        position: relative;
                        width: 140px;
                        height: 140px;
                        flex-shrink: 0;
                    ",
                    div {
                        style: format!(
                            "width: 140px; height: 140px; border-radius: 50%; background: linear-gradient(135deg, #1a1510 0%, #2c2416 100%); display: flex; align-items: center; justify-content: center; box-shadow: 0 8px 24px rgba(0,0,0,0.5); position: relative; border: 3px solid var(--primary); overflow: hidden; transition: transform 0.6s cubic-bezier(0.25, 0.46, 0.45, 0.94);{}",
                            if *is_playing.read() { " animation: vinylSpin 8s linear infinite;" } else { "" }
                        ),
                        if let Some(song) = current_song.read().as_ref() {
                            if let Some(cover_url) = song.cover_url.as_ref() {
                                img {
                                    src: cover_url,
                                    style: "
                                        width: 100%; height: 100%;
                                        border-radius: 50%;
                                        object-fit: cover;
                                        position: absolute;
                                        top: 0; left: 0;
                                    "
                                }
                            } else {
                                div {
                                    style: "
                                        width: 50px; height: 50px; border-radius: 50%;
                                        background: var(--primary); display: flex; align-items: center;
                                        justify-content: center; border: 2px solid var(--secondary);
                                    ",
                                    span { style: "font-size: 20px;", "♫" }
                                }
                                div {
                                    style: "
                                        position: absolute; top: 0; left: 0; right: 0; bottom: 0;
                                        border-radius: 50%;
                                        background: repeating-radial-gradient(circle at center, transparent 0px, transparent 2px, rgba(212,165,116,0.05) 2px, rgba(212,165,116,0.05) 4px);
                                        pointer-events: none;
                                    "
                                }
                            }
                        } else {
                            div {
                                style: "
                                    width: 50px; height: 50px; border-radius: 50%;
                                    background: var(--primary); display: flex; align-items: center;
                                    justify-content: center; border: 2px solid var(--secondary);
                                ",
                                span { style: "font-size: 20px; opacity: 0.5;", "○" }
                            }
                        }
                        div {
                            style: "
                                position: absolute;
                                width: 30px; height: 30px;
                                border-radius: 50%;
                                background: radial-gradient(circle at 45% 45%, #3a2a18 0%, #1a1510 60%, #0f0c08 100%);
                                border: 2px solid rgba(212,165,116,0.2);
                                top: 50%; left: 50%;
                                transform: translate(-50%, -50%);
                                z-index: 2;
                                box-shadow: inset 0 1px 3px rgba(0,0,0,0.5);
                            ",
                            div {
                                style: "
                                    position: absolute;
                                    width: 6px; height: 6px;
                                    border-radius: 50%;
                                    background: radial-gradient(circle at 40% 40%, #8b6914 0%, #5a4520 100%);
                                    top: 50%; left: 50%;
                                    transform: translate(-50%, -50%);
                                "
                            }
                        }
                    }
                    div {
                        style: if *is_playing.read() {
                            "position: absolute; top: -4px; right: 4px; z-index: 20; transform-origin: 82% 5%; transform: rotate(22deg); transition: transform 0.6s cubic-bezier(0.4, 0.0, 0.2, 1);"
                        } else {
                            "position: absolute; top: -4px; right: 4px; z-index: 20; transform-origin: 82% 5%; transform: rotate(-18deg); transition: transform 0.6s cubic-bezier(0.4, 0.0, 0.2, 1);"
                        },
                        div {
                            style: "
                                width: 10px; height: 10px;
                                border-radius: 50%;
                                background: radial-gradient(circle at 40% 40%, #c4a060 0%, #8b6914 50%, #5a4520 100%);
                                border: 1px solid #a08050;
                                box-shadow: 0 1px 4px rgba(0,0,0,0.6);
                                position: relative;
                                z-index: 22;
                            "
                        }
                        div {
                            style: "
                                position: absolute;
                                top: 4px; left: 2px;
                                width: 4px; height: 70px;
                                background: linear-gradient(90deg, #b8956a 0%, #d4a574 30%, #a08050 70%, #8b6914 100%);
                                border-radius: 1px;
                                box-shadow: 1px 1px 4px rgba(0,0,0,0.4);
                                transform: rotate(-5deg);
                                transform-origin: top center;
                            "
                        }
                        div {
                            style: "
                                position: absolute;
                                top: 68px; left: -2px;
                                width: 55px; height: 5px;
                                background: linear-gradient(180deg, #c4a060 0%, #8b6914 50%, #a08050 100%);
                                border-radius: 1px 4px 4px 1px;
                                box-shadow: 1px 1px 4px rgba(0,0,0,0.3);
                                transform: rotate(2deg);
                                transform-origin: left center;
                            "
                        }
                        div {
                            style: "
                                position: absolute;
                                top: 70px; left: -12px;
                                width: 7px; height: 7px;
                                background: radial-gradient(circle at 40% 40%, #d4a574 0%, #8b6914 100%);
                                border-radius: 50%;
                                box-shadow: 0 1px 3px rgba(0,0,0,0.4);
                            "
                        }
                    }
                }
                // Song info
                div {
                    style: "display: flex; flex-direction: column; gap: 8px; min-width: 0;",
                    if let Some(song) = current_song.read().as_ref() {
                        div {
                            style: "font-size: 22px; font-weight: 600; color: var(--text-primary); font-family: var(--font-family); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                            "{song.title}"
                        }
                        div {
                            style: "font-size: 16px; color: var(--text-secondary); font-family: var(--font-family);",
                            "{song.artist}"
                        }
                        div {
                            style: "font-size: 13px; color: var(--text-secondary); font-family: var(--font-family);",
                            "时长：{fmt_time(song.duration_secs)}"
                        }
                    } else {
                        div {
                            style: "font-size: 20px; color: var(--text-secondary); font-family: var(--font-family);",
                            "等待播放"
                        }
                        div {
                            style: "font-size: 14px; color: var(--text-secondary); font-family: var(--font-family);",
                            "请选择一张唱片"
                        }
                    }
                }
            }
            // Scrolling lyrics section
            div {
                id: "lyrics-container-vintage",
                style: "
                    flex: 1;
                    background: rgba(0,0,0,0.2);
                    border: 1px solid rgba(212,165,116,0.15);
                    border-radius: 6px;
                    padding: 16px;
                    overflow-y: auto;
                    mask-image: linear-gradient(transparent 0%, black 8%, black 92%, transparent 100%);
                    position: relative;
                ",
                div {
                    style: "
                        position: absolute;
                        top: 0; left: 0; right: 0; height: 1px;
                        background: linear-gradient(90deg, transparent, rgba(212,165,116,0.2), transparent);
                    "
                }
                if let Some((lines, active_idx_val)) = active_text {
                    for (i, (_, text)) in lines.iter().enumerate() {
                        div {
                            key: "{i}",
                            id: format!("lyric-line-vintage-{}", i),
                            style: format!(
                                "line-height: 2.0; padding: 6px 16px; color: {}; font-family: var(--font-family); font-size: {}; transition: all 0.4s ease-out; opacity: {}; font-weight: {}; position: relative; z-index: 1; border-left: {};",
                                if i == active_idx_val { "var(--text-primary)" } else { "var(--text-secondary)" },
                                if i == active_idx_val { "16px" } else { "13px" },
                                if i == active_idx_val { "1" } else {
                                    let dist = if active_idx_val > i { active_idx_val - i } else { i - active_idx_val };
                                    if dist <= 1 { "0.65" } else if dist <= 3 { "0.4" } else { "0.2" }
                                },
                                if i == active_idx_val { "600" } else { "400" },
                                if i == active_idx_val { "2px solid var(--primary)" } else { "2px solid transparent" }
                            ),
                            "{text}"
                        }
                    }
                } else {
                    div {
                        style: "
                            color: rgba(212,165,116,0.3);
                            font-size: 14px;
                            text-align: center;
                            padding: 40px 20px;
                            font-family: var(--font-family);
                            position: relative;
                            z-index: 1;
                        ",
                        "— 暂无歌词 —"
                    }
                }
            }
        }
    }
}

// ==================== PlayerBar ====================
#[derive(Props, PartialEq, Clone)]
struct PlayerBarProps {
    pub player: Signal<Player>,
    pub current_song: Signal<Option<Song>>,
    pub is_playing: Signal<bool>,
    pub progress: Signal<f64>,
    pub volume: Signal<f64>,
    pub on_toggle_play: Callback<()>,
    pub on_seek: Callback<f64>,
    pub on_volume_change: Callback<f64>,
    pub playback_mode: Signal<PlaybackMode>,
    pub is_seeking: Signal<bool>,
    pub output_devices: Signal<Vec<String>>,
}

#[component]
fn PlayerBar(props: PlayerBarProps) -> Element {
    let current_song = props.current_song;
     let mut is_playing = props.is_playing.clone();
    let mut progress = props.progress;
    let volume = props.volume;
    let on_toggle_play = props.on_toggle_play;
    let on_seek = props.on_seek;
    let on_volume_change = props.on_volume_change;
    let playback_mode = props.playback_mode.clone();
    let mut is_seeking = props.is_seeking;

    let cycle_mode = {
        let mut playback_mode = playback_mode.clone();
        let player = props.player.clone();
        Callback::new(move |_| {
            let current = playback_mode();
            let next = match current {
                PlaybackMode::Sequential => PlaybackMode::LoopOne,
                PlaybackMode::LoopOne => PlaybackMode::LoopAll,
                PlaybackMode::LoopAll => PlaybackMode::Shuffle,
                PlaybackMode::Shuffle => PlaybackMode::Sequential,
            };
            playback_mode.set(next);
            player().set_playback_mode(next);
        })
    };

    let select_device = {
        let player = props.player.clone();
        let current_song = props.current_song.clone();
        move |evt: Event<FormData>| {
            let val = evt.value();
            let device_name = if val == "__default__" { None } else { Some(val) };
            player().set_output_device(device_name);
            if let Some(song) = &*current_song.read() {
                let current_secs = player().progress_secs();
                let _ = player().play(&song.file_path);
                if current_secs > 0.0 {
                    let _ = player().seek(current_secs);
                }
            }
        }
    };

    let player = props.player.clone();
    let current_song = props.current_song.clone();
    let duration = current_song.read().as_ref().map(|s| s.duration_secs).unwrap_or(0.0);
    let progress_val = *progress.read();
    let current_sec = (progress_val / 100.0) * duration;
    let volume_val = *volume.read();
     // For static styling, no reactive spectrum needed in this simplified version

     rsx! {
          div {
              style: "
                  height: 100px;
                  background: linear-gradient(180deg, rgba(30,24,16,1) 0%, rgba(44,36,22,1) 100%);
                  padding: 14px 24px;
                  display: flex;
                  align-items: center;
                  gap: 20px;
                  position: relative;
                  box-shadow: 0 -2px 8px rgba(0,0,0,0.3), inset 0 1px 0 rgba(212,165,116,0.1);
                  overflow: hidden;
              ",
              div {
                  style: "
                      position: absolute;
                      top: 0; left: 0; right: 0; bottom: 0;
                      background: repeating-linear-gradient(
                          90deg,
                          transparent,
                          transparent 2px,
                          rgba(255,255,255,0.015) 2px,
                          rgba(255,255,255,0.015) 4px
                      );
                      pointer-events: none;
                  "
              }
              div {
                  style: "
                      position: absolute;
                      top: 0; left: 0; right: 0; height: 1px;
                      background: linear-gradient(90deg, transparent, rgba(212,165,116,0.25), transparent);
                  "
              }
             div {
                 style: "width: 200px; display: flex; flex-direction: column; gap: 4px;",
                 { if let Some(song) = current_song.read().as_ref() {
                     let title = song.title.clone();
                     let artist = song.artist.clone();
                     rsx! {
                         span {
                             style: "font-size: 15px; font-weight: 600; color: var(--text-primary); font-family: var(--font-family); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                             "{title}"
                         }
                         span {
                             style: "font-size: 13px; color: var(--text-secondary); font-family: var(--font-family);",
                             "{artist}"
                         }
                     }
                 } else {
                     rsx! {
                         span {
                             style: "font-size: 15px; color: rgba(212,165,116,0.3); font-family: var(--font-family);",
                             "未选择唱片"
                         }
                     }
                 } }
             }
              div {
                  style: "flex: 1; display: flex; flex-direction: column; gap: 6px;",
                  div {
                      style: "display: flex; align-items: center; gap: 14px;",
                       button {
                           onclick: move |_| on_toggle_play.call(()),
                           style: "
                               width: 48px;
                               height: 48px;
                               border-radius: 50%;
                               background: radial-gradient(circle at 30% 30%, var(--accent) 0%, var(--primary) 60%, var(--secondary) 100%);
                               border: 2px solid var(--accent);
                               color: #1a1510;
                               font-size: 18px;
                               cursor: pointer;
                               display: flex;
                               align-items: center;
                               justify-content: center;
                               box-shadow:
                                   inset 0 2px 4px rgba(255,255,255,0.3),
                                   0 4px 12px rgba(0,0,0,0.4),
                                   0 0 10px rgba(212,165,116,0.25);
                               transition: all 0.15s ease-out;
                               position: relative;
                               flex-shrink: 0;
                           ",
                           span {
                               style: "text-shadow: 0 0 2px rgba(0,0,0,0.3);",
                               if *is_playing.read() { "⏸" } else { "▶" }
                           }
                       }
                       button {
                           onclick: cycle_mode,
                           style: "
                               width: 36px;
                               height: 36px;
                               border-radius: 50%;
                               background: rgba(212,165,116,0.15);
                               border: 1px solid #d4a574;
                               color: var(--text-primary);
                               font-size: 14px;
                               cursor: pointer;
                               display: flex;
                               align-items: center;
                               justify-content: center;
                               box-shadow: inset 0 1px 2px rgba(255,255,255,0.2);
                               flex-shrink: 0;
                           ",
                           {
                               let mode = *playback_mode.read();
                               let icon = match mode {
                                   PlaybackMode::Sequential => "⏮",
                                   PlaybackMode::LoopOne => "🔂",
                                   PlaybackMode::LoopAll => "🔁",
                                   PlaybackMode::Shuffle => "🔀",
                               };
                               rsx! { span { "{icon}" } }
                           }
                       }
                     div {
                         style: "flex: 1; display: flex; align-items: center; gap: 10px;",
                         span {
                             style: "font-size: 12px; color: var(--text-secondary); font-family: var(--font-family); min-width: 42px; font-variant-numeric: tabular-nums;",
                             "{fmt_time(current_sec)}"
                         }
                         input {
                             r#type: "range",
                             min: "0",
                             max: "100",
                             value: "{progress_val}",
                             oninput: move |e| {
                                 if let Ok(val) = e.value().parse::<f64>() {
                                     is_seeking.set(true);
                                     progress.set(val);
                                 }
                             },
                             onchange: move |e| {
                                 if let Ok(val) = e.value().parse::<f64>() {
                                     on_seek.call(val);
                                     is_seeking.set(false);
                                 }
                             },
                             style: "flex: 1; height: 6px; -webkit-appearance: none; background: rgba(212,165,116,0.2); border-radius: 3px; outline: none; box-shadow: inset 0 1px 3px rgba(0,0,0,0.3);"
                         }
                         span {
                             style: "font-size: 12px; color: var(--text-secondary); font-family: var(--font-family); min-width: 42px; font-variant-numeric: tabular-nums;",
                             "{fmt_time(duration)}"
                         }
                     }
                 }
                div {
                    style: "display: flex; align-items: center; gap: 8px; padding-left: 62px;",
                    span {
                        style: "font-size: 12px; color: rgba(212,165,116,0.5); font-family: var(--font-family);",
                        "音量"
                    }
                    input {
                        r#type: "range",
                        min: "0",
                        max: "100",
                        value: "{(volume_val * 100.0) as i32}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<f64>() {
                                on_volume_change.call(val / 100.0);
                            }
                        },
                        style: "flex: 1; height: 6px; -webkit-appearance: none; background: rgba(212,165,116,0.2); border-radius: 3px; outline: none; box-shadow: inset 0 1px 3px rgba(0,0,0,0.3);"
                    }
                    span {
                        style: "font-size: 12px; color: var(--text-secondary); font-family: var(--font-family); width: 36px; text-align: right;",
                        {format!("{:.0}%", volume_val * 100.0)}
                    }
                    // 设备选择
                    select {
                        onchange: select_device,
                        style: "
                            font-size: 11px;
                            padding: 2px 4px;
                            border: 1px solid var(--secondary);
                            background: rgba(212,165,116,0.1);
                            color: var(--text-secondary);
                            cursor: pointer;
                            font-family: var(--font-family);
                            max-width: 140px;
                            overflow: hidden;
                            text-overflow: ellipsis;
                        ",
                        {
                            let devices = props.output_devices.read();
                            let current = player().output_device();
                            rsx! {
                                option {
                                    value: "__default__",
                                    selected: current.is_none(),
                                    "🎧 默认设备"
                                }
                                for dev in devices.iter().cloned() {
                                    option {
                                        value: "{dev}",
                                        selected: current.as_ref() == Some(&dev),
                                        "{dev}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ==================== VUMeter ====================
 #[derive(Props, PartialEq, Clone)]
 struct VUMeterProps {
     pub spectrum: Signal<[f32; 64]>,
 }
  
    #[component]
    fn VUMeter(props: VUMeterProps) -> Element {
        let spectrum_sig = props.spectrum.clone();
        let bands_snapshot = spectrum_sig.read();
        let bands: [f32; 64] = *bands_snapshot;
        
        // Scale markings positions (left to right percentages)
        let _scale_marks = ["∞", "-20", "-10", "-6", "-3", "0"];
        
        rsx! {
            div {
                style: "
                    width: 100%;
                    height: 90px;
                    display: flex;
                    flex-direction: column;
                    background: linear-gradient(180deg, #3d2b1f 0%, #2c1810 50%, #1a0f08 100%);
                    border: 2px solid #d4a574;
                    border-radius: 8px;
                    box-shadow: 
                        inset 0 2px 8px rgba(0,0,0,0.6),
                        0 0 12px rgba(212,165,116,0.2),
                        0 4px 16px rgba(0,0,0,0.4);
                    padding: 8px 16px;
                    position: relative;
                    overflow: hidden;
                ",
                // Wood grain texture overlay
                div {
                    style: "
                        position: absolute;
                        top: 0; left: 0; right: 0; bottom: 0;
                        background: repeating-linear-gradient(
                            90deg,
                            transparent 0px,
                            transparent 3px,
                            rgba(0,0,0,0.08) 3px,
                            rgba(0,0,0,0.08) 6px
                        ),
                        repeating-linear-gradient(
                            180deg,
                            transparent 0px,
                            transparent 8px,
                            rgba(139,90,43,0.15) 8px,
                            rgba(139,90,43,0.15) 10px
                        );
                        pointer-events: none;
                    "
                }
                // Inner gold frame
                div {
                    style: "
                        position: absolute;
                        top: 4px; left: 4px; right: 4px; bottom: 4px;
                        border: 1px solid rgba(212,165,116,0.3);
                        border-radius: 4px;
                        pointer-events: none;
                    "
                }
                // Top label bar
                div {
                    style: "
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        margin-bottom: 6px;
                        position: relative;
                    ",
                    span {
                        style: "
                            font-family: Georgia, 'Times New Roman', serif;
                            font-size: 11px;
                            color: #d4a574;
                            letter-spacing: 4px;
                            text-transform: uppercase;
                            text-shadow: 0 0 8px rgba(212,165,116,0.5);
                        ",
                        "VINTAGE AUDIO ANALYZER"
                    }
                }
                // Scale markings row
                div {
                    style: "
                        display: flex;
                        justify-content: flex-end;
                        align-items: center;
                        padding-right: 8px;
                        margin-bottom: 4px;
                        position: relative;
                    ",
                    span {
                        style: "
                            font-family: Georgia, 'Times New Roman', serif;
                            font-size: 9px;
                            color: #d4a574;
                            text-shadow: 0 0 4px rgba(212,165,116,0.4);
                        ",
                        "∞    -20   -10   -6   -3   0"
                    }
                }
                // Main meter area
                div {
                    style: "
                        flex: 1;
                        display: flex;
                        gap: 1px;
                        align-items: flex-end;
                        position: relative;
                    ",
                    // Tick marks background
                    div {
                        style: "
                            position: absolute;
                            top: 0; left: 0; right: 0; bottom: 0;
                            background-image: 
                                repeating-linear-gradient(
                                    to right,
                                    transparent,
                                    transparent 7px,
                                    rgba(212,165,116,0.25) 7px,
                                    rgba(212,165,116,0.25) 8px
                                );
                            pointer-events: none;
                        "
                    }
                    for i in 0..64 {
                        div {
                            key: "{i}",
                            style: {
                                let h = 4.0 + bands[i] * 68.0;
                                let is_cluster_sep = (i > 0 && i % 8 == 0) as i32;
                                format!(
                                    "flex: 1; min-width: 3px; height: {}%; transition: height 0.2s ease-in-out; position: relative; margin-left: {}px;",
                                    h,
                                    if is_cluster_sep > 0 { 2 } else { 0 }
                                )
                            },
                            // Track background (shows max range)
                            div {
                                style: "
                                    position: absolute;
                                    bottom: 0; left: 0; right: 0;
                                    height: 100%;
                                    background: rgba(0,0,0,0.4);
                                    border-radius: 1px;
                                "
                            }
                            // Bar body with gradient
                            div {
                                style: format!(
                                    "position: absolute; bottom: 0; left: 0; right: 0; height: {}%; background: linear-gradient(to top, transparent 60%, rgba(212,165,116,0.5) 80%, rgba(255,204,122,0.9) 95%, #ffcc7a 100%); border-radius: 1px 1px 2px 2px; box-shadow: {};",
                                    (bands[i] * 100.0) as i32,
                                    if bands[i] > 0.1 { "0 0 3px rgba(255,204,122,0.4)" } else { "none" }
                                )
                            }
                        }
                    }
                }
                // Bottom tick marks row
                div {
                    style: "
                        display: flex;
                        justify-content: space-between;
                        padding: 0 4px;
                        margin-top: 4px;
                    ",
                    span {
                        style: "
                            font-family: Georgia, serif;
                            font-size: 8px;
                            color: rgba(212,165,116,0.6);
                        ",
                        "L"
                    }
                    span {
                        style: "
                            font-family: Georgia, serif;
                            font-size: 8px;
                            color: rgba(212,165,116,0.6);
                        ",
                        "R"
                    }
                }
                // LED indicator dots between clusters
                div {
                    style: "
                        position: absolute;
                        top: 50%;
                        left: 50%;
                        transform: translate(-50%, -50%);
                        display: flex;
                        gap: 28px;
                    ",
                    // Amber LED indicators
                    for led_i in 0..6 {
                        div {
                            key: "led-{led_i}",
                            style: "
                                width: 4px;
                                height: 4px;
                                border-radius: 50%;
                                background: rgba(255,180,100,0.7);
                                box-shadow: 0 0 6px rgba(255,180,100,0.5);
                            "
                        }
                    }
                }
                // Cluster separator lines (gold vertical lines every 8 bands)
                div {
                    style: "
                        position: absolute;
                        top: 20%; bottom: 20%; left: 0; right: 0;
                        background-image: 
                            repeating-linear-gradient(
                                to right,
                                transparent,
                                transparent calc(12.5% - 2px),
                                rgba(212,165,116,0.4) calc(12.5% - 2px),
                                rgba(212,165,116,0.4) calc(12.5% - 1px),
                                transparent calc(12.5% - 1px),
                                transparent calc(25% - 2px),
                                rgba(212,165,116,0.4) calc(25% - 2px),
                                rgba(212,165,116,0.4) calc(25% - 1px),
                                transparent calc(25% - 1px),
                                transparent calc(37.5% - 2px),
                                rgba(212,165,116,0.4) calc(37.5% - 2px),
                                rgba(212,165,116,0.4) calc(37.5% - 1px),
                                transparent calc(37.5% - 1px),
                                transparent calc(50% - 2px),
                                rgba(212,165,116,0.4) calc(50% - 2px),
                                rgba(212,165,116,0.4) calc(50% - 1px),
                                transparent calc(50% - 1px),
                                transparent calc(62.5% - 2px),
                                rgba(212,165,116,0.4) calc(62.5% - 2px),
                                rgba(212,165,116,0.4) calc(62.5% - 1px),
                                transparent calc(62.5% - 1px),
                                transparent calc(75% - 2px),
                                rgba(212,165,116,0.4) calc(75% - 2px),
                                rgba(212,165,116,0.4) calc(75% - 1px),
                                transparent calc(75% - 1px),
                                transparent calc(87.5% - 2px),
                                rgba(212,165,116,0.4) calc(87.5% - 2px),
                                rgba(212,165,116,0.4) calc(87.5% - 1px),
                                transparent calc(87.5% - 1px)
                            );
                        pointer-events: none;
                    "
                }
            }
        }
    }

// ==================== App ====================
#[component]
pub fn App(
    theme: Signal<ThemeManager>,
    player: Signal<Player>,
    db: Signal<Database>,
    current_song: Signal<Option<Song>>,
    is_playing: Signal<bool>,
    progress: Signal<f64>,
    progress_secs: Signal<f64>,
    volume: Signal<f64>,
    lyrics: Signal<Option<Lyrics>>,
    on_import: Callback<()>,
    spectrum: Signal<[f32; 64]>,
    refresh_songs: Signal<u32>,
    playback_mode: Signal<PlaybackMode>,
    is_seeking: Signal<bool>,
    output_devices: Signal<Vec<String>>,
) -> Element {
    let mut is_playing = is_playing.clone();
    let toggle_play = move |_| {
        if *is_playing.read() {
            player().pause();
            is_playing.set(false);
        } else if let Some(song) = &*current_song.read() {
            let _ = player().play(&song.file_path);
            is_playing.set(true);
        }
    };

    let seek = move |val: f64| {
        let duration = current_song.read().as_ref().map(|s| s.duration_secs).unwrap_or(0.0);
        let seconds = (val / 100.0) * duration;
        let _ = player().seek(seconds);
    };

    let set_volume = move |val: f64| {
        let db_val = if val > 0.0 { (val - 1.0) * 20.0 } else { -60.0 };
        player().set_volume_db(db_val);
        volume.set(val);
    };

      let select_song = move |song: Song| {
          current_song.set(Some(song.clone()));
          match player().play(&song.file_path) {
              Ok(_) => {
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
              Err(e) => {
                  eprintln!("Failed to play {}: {}", song.title, e);
                  is_playing.set(false);
              }
          }
      };

     rsx! {
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
             TitleBar { theme: theme.clone(), on_import: on_import.clone() }
             div {
                     style: "
                         flex: 1;
                         display: flex;
                         overflow: hidden;
                     ",
                 Sidebar {
                     db: db.clone(),
                     current_song: current_song.clone(),
                     on_select: Callback::new(select_song),
                     refresh_songs: refresh_songs.clone(),
                 }
                 MainArea {
                     current_song: current_song.clone(),
                     lyrics: lyrics.clone(),
                     progress_secs: progress_secs.clone(),
                     is_playing: is_playing.clone(),
                 }
             }
             VUMeter { spectrum: spectrum.clone() }
             PlayerBar {
                 player: player.clone(),
                 current_song: current_song.clone(),
                 is_playing: is_playing.clone(),
                 progress: progress.clone(),
                 volume: volume.clone(),
                 on_toggle_play: Callback::new(toggle_play),
                 on_seek: Callback::new(seek),
                 on_volume_change: Callback::new(set_volume),
                 playback_mode: playback_mode.clone(),
                 is_seeking: is_seeking.clone(),
                 output_devices: output_devices.clone(),
             }
         }
     }
}