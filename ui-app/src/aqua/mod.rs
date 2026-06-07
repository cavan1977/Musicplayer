//! Aqua Glass Theme - 水琉璃风格
//! 横向三列布局：左列(大封面+歌曲信息) | 中列(歌词) | 右列(歌曲列表)
//! 完全有机流体、玻璃拟态、大圆角、无边框、全中文

use dioxus::prelude::*;
use dioxus_desktop::use_window;
use db::Database;
use audio::Player;
use audio::backend::PlaybackState;
use audio::PlaybackMode;
use crate::{Song, format_time, UiStyle, ThemeManager};
use crate::lyrics_parser::Lyrics;

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
                 height: 48px;
                 background: rgba(15,12,41,0.6);
                 backdrop-filter: blur(28px) saturate(1.2);
                 display: flex;
                 align-items: center;
                 justify-content: space-between;
                 padding: 0 20px;
                 box-shadow: 0 1px 0 rgba(255,255,255,0.04), 0 2px 12px rgba(0,0,0,0.15);
                 user-select: none;
                 -webkit-app-region: drag;
                 position: relative;
                 overflow: hidden;
                 z-index: 100;
             ",
             div {
                 style: "
                     position: absolute;
                     top: 0; left: 0; right: 0; bottom: 0;
                     background: linear-gradient(90deg, rgba(102,126,234,0.05) 0%, transparent 30%, transparent 70%, rgba(118,75,162,0.05) 100%);
                     pointer-events: none;
                 "
             }
             div {
                 style: "display: flex; align-items: center; gap: 12px; -webkit-app-region: no-drag; z-index: 1;",
                 div {
                     style: "
                         width: 28px; height: 28px;
                         background: linear-gradient(135deg, #667eea, #764ba2);
                         border-radius: 8px;
                         display: flex; align-items: center; justify-content: center;
                         box-shadow: 0 2px 8px rgba(102,126,234,0.4);
                     ",
                     span { style: "font-size: 14px; color: white;", "♪" }
                 }
                 span {
                     style: "
                         font-family: var(--font-family);
                         font-size: 15px;
                         font-weight: 600;
                         color: var(--text-primary);
                         letter-spacing: 0.5px;
                     ",
                     "水琉璃音乐"
                 }
             }
             div {
                 style: "display: flex; align-items: center; gap: 4px; -webkit-app-region: no-drag; z-index: 1;",
                 ThemeButton {
                     text: "水琉璃".to_string(),
                     active: true,
                     on_click: Callback::new(move |_| switch(UiStyle::AquaGlass))
                 }
                 ThemeButton {
                     text: "声波流".to_string(),
                     active: false,
                     on_click: Callback::new(move |_| switch(UiStyle::SonicFlux))
                 }
                 ThemeButton {
                     text: "经典".to_string(),
                     active: false,
                     on_click: Callback::new(move |_| switch(UiStyle::VintagePro))
                 }
                 div {
                     style: "width: 1px; height: 20px; background: rgba(255,255,255,0.1); margin: 0 6px;"
                 }
                 button {
                     onclick: move |_| on_import.call(()),
                     style: "
                         padding: 5px 14px;
                         border: 1px solid rgba(100,255,218,0.3);
                         background: rgba(100,255,218,0.08);
                         color: var(--color-accent);
                         cursor: pointer;
                         font-size: 12px;
                         font-family: var(--font-family);
                         border-radius: 6px;
                         transition: all 0.2s ease;
                     ",
                     "导入"
                 }
                 button {
                     onclick: move |_| window_min.set_minimized(true),
                     style: "
                         width: 28px;
                         height: 28px;
                         border-radius: 8px;
                         border: none;
                         background: rgba(255,255,255,0.06);
                         color: var(--text-secondary);
                         cursor: pointer;
                         font-size: 14px;
                         transition: background 0.2s;
                     ",
                     "−"
                 }
                 button {
                     onclick: move |_| window_close.close(),
                     style: "
                         width: 28px;
                         height: 28px;
                         border-radius: 8px;
                         border: none;
                         background: rgba(255,59,48,0.8);
                         color: white;
                         cursor: pointer;
                         font-size: 12px;
                         transition: all 0.2s;
                     ",
                     "✕"
                 }
             }
         }
     }
 }

// ==================== ThemeButton ====================
#[derive(Props, PartialEq, Clone)]
struct ThemeButtonProps {
    pub text: String,
    pub active: bool,
    pub on_click: EventHandler<MouseEvent>,
}

#[component]
fn ThemeButton(props: ThemeButtonProps) -> Element {
    let active = props.active;
    let on_click = props.on_click;

    rsx! {
        button {
            onclick: move |evt| on_click.call(evt),
            style: format!(
                "padding: 8px 18px; border-radius: var(--radius-sm); color: {}; font-size: 13px; cursor: pointer; transition: all 0.2s; background: {}; box-shadow: {}; border: none;",
                if active { "var(--text-primary)" } else { "var(--text-secondary)" },
                if active { "rgba(255,255,255,0.18)" } else { "rgba(255,255,255,0.06)" },
                if active { "0 2px 8px rgba(102,126,234,0.3)" } else { "none" }
            ),
            "{props.text}"
        }
    }
}

// ==================== CoverColumn (左列：大封面+信息) ====================
#[derive(Props, PartialEq, Clone)]
struct CoverColumnProps {
    pub current_song: Signal<Option<Song>>,
    pub is_playing: Signal<bool>,
}

#[component]
fn CoverColumn(props: CoverColumnProps) -> Element {
    let current_song = props.current_song;
    let is_playing = props.is_playing;
    let playing = *is_playing.read();

    rsx! {
        div {
            style: "
                flex: 0 0 300px;
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 16px;
                padding: 16px;
                min-width: 0;
            ",
            // 大封面 - 圆形唱片
            div {
                style: "
                    position: relative;
                    width: 240px;
                    height: 240px;
                    flex-shrink: 0;
                ",
                div {
                    style: "
                        width: 240px; height: 240px;
                        border-radius: 50%;
                        display: flex; align-items: center; justify-content: center;
                        box-shadow:
                            0 16px 48px rgba(0,0,0,0.4),
                            0 0 60px rgba(102,126,234,0.15),
                            inset 0 0 60px rgba(255,255,255,0.02);
                        flex-shrink: 0;
                        overflow: hidden;
                        position: relative;
                    ",
                    div {
                        style: "
                            position: absolute;
                            top: -2px; left: -2px; right: -2px; bottom: -2px;
                            border-radius: 50%;
                            background: conic-gradient(
                                from 0deg,
                                var(--accent),
                                var(--primary),
                                var(--secondary),
                                var(--accent)
                            );
                            animation: rotate 8s linear infinite;
                            opacity: 0.3;
                            filter: blur(8px);
                            z-index: -1;
                        "
                    }
                    if let Some(song) = current_song.read().as_ref() {
                        if let Some(cover_url) = song.cover_url.as_ref() {
                           img {
                               src: cover_url,
                               style: format!(
                                   "width: 100%; height: 100%; object-fit: cover; filter: saturate(1.1) contrast(1.05); transition: transform 0.8s cubic-bezier(0.25, 0.46, 0.45, 0.94);{}",
                                   if playing { " animation: vinylSpin 8s linear infinite;" } else { "" }
                               )
                           }
                        } else {
                           div {
                               style: "
                                   width: 100%; height: 100%;
                                   border-radius: 50%;
                                   background: linear-gradient(135deg, var(--primary), var(--secondary));
                                   display: flex; align-items: center; justify-content: center;
                                   position: relative;
                               ",
                               div {
                                   style: "
                                       position: absolute;
                                       top: 0; left: 0; right: 0; bottom: 0;
                                       border-radius: 50%;
                                       background: linear-gradient(
                                           45deg,
                                           rgba(255,255,255,0) 40%,
                                           rgba(255,255,255,0.1) 50%,
                                           rgba(255,255,255,0) 60%
                                       );
                                       animation: shimmer 3s ease-in-out infinite;
                                   "
                               }
                               div {
                                   style: "font-size: 50px; color: rgba(255,255,255,0.5); z-index: 1;",
                                   "♪"
                               }
                           }
                       }
                   } else {
                       div {
                           style: "
                               width: 100%; height: 100%;
                               border-radius: 50%;
                               background: linear-gradient(135deg, var(--primary), var(--secondary));
                               display: flex; align-items: center; justify-content: center;
                           ",
                           div {
                               style: "font-size: 50px; color: rgba(255,255,255,0.4);",
                               "♫"
                           }
                       }
                   }
                   // 唱片中心
                   div {
                       style: "
                           position: absolute;
                           width: 56px; height: 56px;
                           border-radius: 50%;
                           background: radial-gradient(circle at 45% 45%, #2a2a4a 0%, #1a1a2e 40%, #0f0c29 100%);
                           border: 3px solid rgba(255,255,255,0.12);
                           top: 50%; left: 50%;
                           transform: translate(-50%, -50%);
                           z-index: 2;
                           box-shadow: inset 0 2px 6px rgba(0,0,0,0.6), 0 0 12px rgba(0,0,0,0.3);
                       ",
                       div {
                           style: "
                               position: absolute;
                               width: 10px; height: 10px;
                               border-radius: 50%;
                               background: radial-gradient(circle at 40% 40%, #666 0%, #333 100%);
                               border: 1px solid rgba(255,255,255,0.2);
                               top: 50%; left: 50%;
                               transform: translate(-50%, -50%);
                           "
                       }
                   }
                }
                // 唱针
                div {
                    style: if playing {
                        "position: absolute; top: -6px; right: 8px; z-index: 20; transform-origin: 85% 5%; transform: rotate(25deg); transition: transform 0.6s cubic-bezier(0.4, 0.0, 0.2, 1);"
                    } else {
                        "position: absolute; top: -6px; right: 8px; z-index: 20; transform-origin: 85% 5%; transform: rotate(-20deg); transition: transform 0.6s cubic-bezier(0.4, 0.0, 0.2, 1);"
                    },
                    div {
                        style: "
                            width: 12px; height: 12px;
                            border-radius: 50%;
                            background: radial-gradient(circle at 40% 40%, #aaa 0%, #666 50%, #333 100%);
                            border: 2px solid #777;
                            box-shadow: 0 2px 6px rgba(0,0,0,0.6);
                            position: relative;
                            z-index: 22;
                        "
                    }
                    div {
                        style: "
                            position: absolute;
                            top: 4px; left: 2px;
                            width: 4px; height: 90px;
                            background: linear-gradient(90deg, #888 0%, #bbb 25%, #999 50%, #777 75%, #888 100%);
                            border-radius: 2px;
                            box-shadow: 2px 2px 6px rgba(0,0,0,0.5);
                            transform: rotate(-8deg);
                            transform-origin: top center;
                        "
                    }
                }
            }
            // 歌曲信息卡片
            div {
                style: "
                    width: 100%;
                    background: rgba(255,255,255,0.04);
                    backdrop-filter: blur(20px) saturate(1.1);
                    border: 1px solid rgba(255,255,255,0.08);
                    border-radius: 20px;
                    padding: 20px;
                    box-shadow: 0 8px 32px rgba(0,0,0,0.2), inset 0 1px 0 rgba(255,255,255,0.06);
                    position: relative;
                    overflow: hidden;
                ",
                div {
                    style: "
                        position: absolute;
                        top: 0; left: 0; right: 0; height: 1px;
                        background: linear-gradient(90deg, transparent, rgba(102,126,234,0.3), rgba(100,255,218,0.2), transparent);
                    "
                }
                if let Some(song) = &*current_song.read() {
                    div {
                        style: "font-size: 20px; font-weight: 700; color: var(--text-primary); margin-bottom: 10px; letter-spacing: -0.3px; line-height: 1.3; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{song.title}"
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 8px; margin-bottom: 6px;",
                        div {
                            style: "width: 3px; height: 14px; background: linear-gradient(180deg, var(--color-primary), var(--color-secondary)); border-radius: 2px;"
                        }
                        span { style: "font-size: 14px; color: var(--text-secondary);", "{song.artist}" }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        div {
                            style: "width: 3px; height: 14px; background: linear-gradient(180deg, var(--color-accent), rgba(100,255,218,0.3)); border-radius: 2px;"
                        }
                        span { style: "font-size: 13px; color: rgba(255,255,255,0.4);", "{format_time(song.duration_secs)}" }
                    }
                } else {
                    div { style: "font-size: 16px; color: rgba(255,255,255,0.3); margin-bottom: 6px; font-weight: 500;", "未播放" }
                    div { style: "font-size: 13px; color: rgba(255,255,255,0.2);", "请选择一首歌曲" }
                }
            }
        }
    }
}

// ==================== LyricsSection (中列：歌词) ====================
#[derive(Props, PartialEq, Clone)]
struct LyricsSectionProps {
    pub lyrics: Signal<Option<Lyrics>>,
    pub progress_secs: Signal<f64>,
    pub current_song: Signal<Option<Song>>,
}

#[component]
fn LyricsSection(props: LyricsSectionProps) -> Element {
    let lyrics = props.lyrics;
    let progress_secs = props.progress_secs;

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
                            "var c=document.getElementById('lyrics-container-aqua'); \
                             var l=document.getElementById('lyric-line-{}'); \
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
            id: "lyrics-container-aqua",
            style: "
                flex: 1;
                background: rgba(255,255,255,0.03);
                border: 1px solid rgba(255,255,255,0.06);
                border-radius: 20px;
                padding: 20px;
                overflow-y: auto;
                mask-image: linear-gradient(transparent 0%, black 10%, black 90%, transparent 100%);
                position: relative;
            ",
            div {
                style: "
                    position: absolute;
                    top: 0; left: 0; right: 0; height: 1px;
                    background: linear-gradient(90deg, transparent, rgba(100,255,218,0.15), transparent);
                "
            }
            if let Some((lines, active_idx_val)) = active_text {
                for (i, (_, text)) in lines.iter().enumerate() {
                    div {
                        key: "{i}",
                        id: format!("lyric-line-{}", i),
                        style: format!(
                            "line-height: 2.0; padding: 6px 14px; color: {}; font-size: {}; transition: all 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94); opacity: {}; font-weight: {}; border-radius: 8px; position: relative;",
                            if i == active_idx_val { "var(--text-primary)" } else { "var(--text-secondary)" },
                            if i == active_idx_val { "17px" } else { "14px" },
                            if i == active_idx_val { "1" } else {
                                let dist = if active_idx_val > i { active_idx_val - i } else { i - active_idx_val };
                                if dist <= 1 { "0.7" } else if dist <= 3 { "0.45" } else { "0.25" }
                            },
                            if i == active_idx_val { "600" } else { "400" }
                        ),
                        if i == active_idx_val {
                            div {
                                style: "
                                    position: absolute;
                                    left: 0; top: 50%;
                                    transform: translateY(-50%);
                                    width: 3px; height: 60%;
                                    background: linear-gradient(180deg, var(--color-primary), var(--color-accent));
                                    border-radius: 2px;
                                "
                            }
                        }
                        "{text}"
                    }
                }
            } else {
                div {
                    style: "
                        color: rgba(255,255,255,0.25);
                        font-size: 15px;
                        text-align: center;
                        padding: 40px 20px;
                    ",
                    "— 暂无歌词 —"
                }
            }
        }
    }
}

// ==================== PlaylistPanel (右列：歌曲列表) ====================
#[derive(Props, PartialEq, Clone)]
struct PlaylistPanelProps {
    pub db: Signal<Database>,
    pub on_song_select: Callback<Song>,
    pub current_song: Signal<Option<Song>>,
    pub refresh_songs: Signal<u32>,
}

 #[component]
 fn PlaylistPanel(props: PlaylistPanelProps) -> Element {
     let db_sig = props.db.clone();
     let mut songs: Signal<Vec<Song>> = use_signal(move || db_sig.read().list_songs().unwrap_or_default());
     let on_select = props.on_song_select.clone();
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
                  flex: 0 0 280px;
                  background: rgba(255,255,255,0.03);
                  border: 1px solid rgba(255,255,255,0.06);
                  border-radius: 20px;
                  padding: 20px;
                  overflow-y: auto;
                  position: relative;
              ",
              div {
                  style: "
                      position: absolute;
                      top: 0; left: 0; right: 0; height: 1px;
                      background: linear-gradient(90deg, transparent, rgba(102,126,234,0.2), transparent);
                  "
              }
              h3 {
                  style: "
                      font-size: 16px;
                      font-weight: 600;
                      color: var(--text-primary);
                      margin: 0 0 16px 0;
                      padding-bottom: 12px;
                      border-bottom: 1px solid rgba(255,255,255,0.06);
                      letter-spacing: 0.3px;
                  ",
                  "音乐库"
              }
              if songs_guard.is_empty() {
                  div {
                      style: "
                          color: rgba(255,255,255,0.25);
                          font-size: 14px;
                          padding: 40px 20px;
                          text-align: center;
                      ",
                      "音乐库为空，请添加音乐文件"
                  }
               } else {
                   for song in songs_guard.iter().cloned() {
                       {
                          let song_cb = song.clone();
                          let is_current = current_song.read().as_ref().map(|cs| cs.id == song.id).unwrap_or(false);
                          rsx! {
                              div {
                                  key: "{song.id.unwrap_or(0)}",
                                  onclick: move |_| on_select.call(song_cb.clone()),
                                  style: format!(
                                      "padding: 10px 14px; margin-bottom: 4px; border-radius: 10px; background: {}; cursor: pointer; transition: all 0.2s ease; border-left: {};",
                                      if is_current { "rgba(102,126,234,0.15)" } else { "rgba(255,255,255,0.02)" },
                                      if is_current { "3px solid var(--color-primary)" } else { "3px solid transparent" }
                                  ),
                                  div {
                                      style: format!(
                                          "font-weight: {}; color: {}; font-size: 14px; margin-bottom: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                          if is_current { "600" } else { "500" },
                                          if is_current { "var(--text-primary)" } else { "var(--text-secondary)" }
                                      ),
                                      "{&song.title}"
                                  },
                                  div {
                                      style: "font-size: 12px; color: rgba(255,255,255,0.35); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
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

// ==================== MainPanel ====================
#[derive(Props, PartialEq, Clone)]
struct MainPanelProps {
    pub player: Signal<Player>,
    pub current_song: Signal<Option<Song>>,
    pub is_playing: Signal<bool>,
    pub progress: Signal<f64>,
    pub progress_secs: Signal<f64>,
    pub volume: Signal<f64>,
    pub db: Signal<Database>,
    pub lyrics: Signal<Option<Lyrics>>,
    pub on_song_select: Callback<Song>,
    pub refresh_songs: Signal<u32>,
}

#[component]
fn MainPanel(props: MainPanelProps) -> Element {
    rsx! {
        div {
            style: "
                flex: 1;
                display: flex;
                gap: 16px;
                padding: 16px 20px;
                overflow: hidden;
                min-height: 0;
            ",
            // 左列：封面 + 信息
            CoverColumn {
                current_song: props.current_song.clone(),
                is_playing: props.is_playing.clone(),
            }
            // 中列：歌词
            LyricsSection {
                lyrics: props.lyrics.clone(),
                progress_secs: props.progress_secs.clone(),
                current_song: props.current_song.clone(),
            }
            // 右列：歌曲列表
            PlaylistPanel {
                db: props.db.clone(),
                on_song_select: props.on_song_select.clone(),
                current_song: props.current_song.clone(),
                refresh_songs: props.refresh_songs.clone(),
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
    pub playback_mode: Signal<PlaybackMode>,
    pub is_seeking: Signal<bool>,
    pub output_devices: Signal<Vec<String>>,
}

#[component]
fn PlayerBar(props: PlayerBarProps) -> Element {
    let player = props.player;
    let current_song = props.current_song;
    let mut is_playing = props.is_playing.clone();
    let mut progress = props.progress;
    let mut volume = props.volume;
    let playback_mode = props.playback_mode.clone();
    let mut is_seeking = props.is_seeking;

    let toggle_play = {
        let player = player.clone();
        let mut is_playing = is_playing.clone();
        move |_| {
            let state = player().state();
            match state {
                PlaybackState::Playing => { player().pause(); is_playing.set(false); },
                PlaybackState::Paused | PlaybackState::Stopped => {
                    if let Some(song) = &*current_song.read() {
                        let _ = player().play(song.file_path.clone());
                        is_playing.set(true);
                    }
                }
            }
        }
    };

    let cycle_mode = {
        let mut playback_mode = playback_mode.clone();
        let player = player.clone();
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

    let total_secs = player().duration_secs();
    let elapsed = if total_secs > 0.0 { total_secs * (*progress.read()) / 100.0 } else { 0.0 };
    let volume_val = *volume.read();

    let seek = move |pct: f64| {
        let total = player().duration_secs();
        if total > 0.0 {
            let seconds = total * pct / 100.0;
            let _ = player().seek(seconds);
        }
    };

    rsx! {
        div {
            style: "
                width: 100%;
                height: 96px;
                background: rgba(15,12,41,0.75);
                backdrop-filter: blur(28px) saturate(1.2);
                display: flex;
                align-items: center;
                padding: 0 28px;
                gap: 20px;
                box-shadow: 0 -1px 0 rgba(255,255,255,0.04), 0 -4px 20px rgba(0,0,0,0.3);
                flex-shrink: 0;
             ",
             if let Some(song) = current_song.read().as_ref() {
                 if let Some(cover_url) = song.cover_url.as_ref() {
                     img {
                         src: cover_url,
                         style: "width: 48px; height: 48px; border-radius: 10px; object-fit: cover; flex-shrink: 0; box-shadow: 0 2px 8px rgba(0,0,0,0.3);"
                     }
                 } else {
                     div {
                         style: "
                             width: 48px;
                             height: 48px;
                             border-radius: 10px;
                             background: linear-gradient(135deg, var(--color-primary), var(--color-secondary));
                             display: flex;
                             align-items: center;
                             justify-content: center;
                             font-size: 20px;
                             flex-shrink: 0;
                             box-shadow: 0 2px 8px rgba(0,0,0,0.3);
                         ",
                         "♫"
                     }
                 }
             } else {
                 div {
                     style: "
                         width: 48px;
                         height: 48px;
                         border-radius: 10px;
                         background: rgba(255,255,255,0.06);
                         display: flex;
                         align-items: center;
                         justify-content: center;
                         font-size: 20px;
                         flex-shrink: 0;
                     ",
                     "🎵"
                 }
             }

            div {
                style: "flex: 1; min-width: 0; display: flex; flex-direction: column; justify-content: center;",
                if let Some(song) = &*current_song.read() {
                    div {
                        style: "
                            font-size: 15px;
                            font-weight: 600;
                            color: var(--text-primary);
                            white-space: nowrap;
                            overflow: hidden;
                            text-overflow: ellipsis;
                        ",
                        "{song.title}"
                    }
                    div {
                        style: "
                            font-size: 13px;
                            color: var(--text-secondary);
                            white-space: nowrap;
                            overflow: hidden;
                            text-overflow: ellipsis;
                        ",
                        "{song.artist}"
                    }
                } else {
                    div { style: "font-size: 15px; font-weight: 600; color: var(--text-primary);", "未选择曲目" }
                    div { style: "font-size: 13px; color: var(--text-secondary);", "请从列表选择" }
                }
            }

              div {
                  style: "flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px;",
                  div {
                      style: "display: flex; align-items: center; gap: 14px;",
                      button {
                          onclick: toggle_play,
                          style: "
                              width: 42px;
                              height: 42px;
                              border-radius: 50%;
                              border: none;
                              background: linear-gradient(135deg, #667eea, #764ba2);
                              color: white;
                              cursor: pointer;
                              display: flex;
                              align-items: center;
                              justify-content: center;
                              font-size: 15px;
                              box-shadow: 0 4px 16px rgba(102,126,234,0.5);
                              transition: all 0.2s ease;
                              flex-shrink: 0;
                          ",
                          if *is_playing.read() { "⏸" } else { "▶" }
                      }
                      button {
                          onclick: cycle_mode,
                          style: "
                              width: 32px;
                              height: 32px;
                              border-radius: 50%;
                              border: none;
                              background: rgba(255,255,255,0.1);
                              color: white;
                              cursor: pointer;
                              display: flex;
                              align-items: center;
                              justify-content: center;
                              font-size: 12px;
                              box-shadow: 0 2px 6px rgba(0,0,0,0.2);
                              flex-shrink: 0;
                          ",
                          {
                              let mode = *playback_mode.read();
                              match mode {
                                  PlaybackMode::Sequential => "⏮",
                                  PlaybackMode::LoopOne => "🔂",
                                  PlaybackMode::LoopAll => "🔁",
                                  PlaybackMode::Shuffle => "🔀",
                              }
                          }
                      }
                      div {
                          style: "flex: 1; display: flex; align-items: center; gap: 10px;",
                          span {
                              style: "font-size: 12px; color: var(--text-secondary); font-variant-numeric: tabular-nums; min-width: 38px;",
                              "{format_time(elapsed)}"
                          }
                          input {
                              r#type: "range",
                              min: "0",
                              max: "100",
                              value: "{*progress.read()}",
                              oninput: move |e| {
                                  if let Ok(val) = e.value().parse::<f64>() {
                                      is_seeking.set(true);
                                      progress.set(val);
                                  }
                              },
                              onchange: move |e| {
                                  if let Ok(val) = e.value().parse::<f64>() {
                                      seek(val);
                                      is_seeking.set(false);
                                  }
                              },
                              style: "
                                 -webkit-appearance: none;
                                 flex: 1;
                                 height: 6px;
                                 border-radius: 3px;
                                 background: rgba(255,255,255,0.15);
                                 outline: none;
                             "
                          }
                          span {
                              style: "font-size: 12px; color: var(--text-secondary); font-variant-numeric: tabular-nums; min-width: 38px; text-align: right;",
                              "{format_time(total_secs)}"
                          }
                      }
                  }
                  div {
                      style: "display: flex; align-items: center; gap: 8px; padding-left: 56px;",
                      span {
                          style: "font-size: 12px; color: var(--text-secondary);",
                          "🔊"
                      }
                      input {
                          r#type: "range",
                          min: "0",
                          max: "100",
                          value: "{volume_val * 100.0}",
                          oninput: move |e| {
                              if let Ok(val) = e.value().parse::<f64>() {
                                  let v = val / 100.0;
                                  volume.set(v);
                                  player().set_volume_db((v * 80.0) - 80.0);
                              }
                          },
                          style: "
                             -webkit-appearance: none;
                             flex: 1;
                             height: 6px;
                             border-radius: 3px;
                             background: rgba(255,255,255,0.15);
                             outline: none;
                         "
                      }
                      span {
                          style: "font-size: 12px; color: var(--text-secondary); width: 38px; text-align: right;",
                          {format!("{:.0}%", volume_val * 100.0)}
                      }
                  }
                  // 设备选择
                  div {
                      style: "display: flex; align-items: center; gap: 4px; flex-shrink: 0;",
                      select {
                          onchange: move |evt: Event<FormData>| {
                              let val = evt.value();
                              let device_name = if val == "__default__" {
                                  None
                              } else {
                                  Some(val)
                              };
                              player().set_output_device(device_name);
                              if let Some(song) = &*current_song.read() {
                                  let current_secs = player().progress_secs();
                                  let _ = player().play(&song.file_path);
                                  if current_secs > 0.0 {
                                      let _ = player().seek(current_secs);
                                  }
                                  is_playing.set(true);
                              }
                          },
                          style: "
                              font-size: 11px;
                              padding: 2px 4px;
                              border-radius: 4px;
                              border: 1px solid rgba(255,255,255,0.15);
                              background: rgba(255,255,255,0.06);
                              color: var(--text-secondary);
                              cursor: pointer;
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

 // ==================== VUMeterOverlay ====================
 // 音频可视化：从左侧贯穿到右侧的频谱条
 #[derive(Props, PartialEq, Clone)]
 struct VUMeterOverlayProps {
     pub spectrum: Signal<[f32; 64]>,
 }

 #[component]
 fn VUMeterOverlay(props: VUMeterOverlayProps) -> Element {
     let spectrum_sig = props.spectrum.clone();
     let bands_snapshot = spectrum_sig.read();
     let bands: [f32; 64] = *bands_snapshot;

     rsx! {
         div {
             style: "
                 width: 100%;
                 height: 64px;
                 background: rgba(0,0,0,0.35);
                 backdrop-filter: blur(12px);
                 -webkit-backdrop-filter: blur(12px);
                 display: flex;
                 align-items: flex-end;
                 padding: 0;
                 box-shadow: 0 -2px 16px rgba(0,0,0,0.2), inset 0 1px 0 rgba(100,255,218,0.08);
                 flex-shrink: 0;
                 position: relative;
                 overflow: hidden;
             ",
             // 顶部渐变分隔线 - 贯穿全宽
             div {
                 style: "
                     position: absolute;
                     top: 0;
                     left: 0;
                     right: 0;
                     height: 2px;
                     background: linear-gradient(90deg,
                         rgba(102,126,234,0.3) 0%,
                         rgba(100,255,218,0.5) 25%,
                         rgba(0,255,255,0.6) 50%,
                         rgba(100,255,218,0.5) 75%,
                         rgba(118,75,162,0.3) 100%
                     );
                     z-index: 2;
                 "
             }
             // 64根频谱条均匀铺满全宽，从左到右
             for i in 0..64 {
                 {
                     let band_val = bands[i];
                     let bar_height = (3.0 + band_val * 56.0) as i32;
                     // 颜色从左侧蓝紫渐变到右侧青绿
                     let ratio = i as f64 / 63.0;
                     let r = (102.0 * (1.0 - ratio) + 0.0 * ratio) as u8;
                     let g = (126.0 * (1.0 - ratio) + 255.0 * ratio) as u8;
                     let b = (234.0 * (1.0 - ratio) + 218.0 * ratio) as u8;
                     let r2 = (118.0 * (1.0 - ratio) + 100.0 * ratio) as u8;
                     let g2 = (75.0 * (1.0 - ratio) + 255.0 * ratio) as u8;
                     let b2 = (162.0 * (1.0 - ratio) + 153.0 * ratio) as u8;
                     rsx! {
                         div {
                             key: "bar-{i}",
                             style: format!(
                                 "flex: 1;
                                 height: 56px;
                                 position: relative;
                                 display: flex;
                                 align-items: flex-end;
                                 justify-content: center;
                                 margin: 0 0.5px;
                                 ",
                             ),
                             // 背景条
                             div {
                                 style: "
                                     position: absolute;
                                     bottom: 0;
                                     left: 0;
                                     right: 0;
                                     height: 100%;
                                     background: rgba(255,255,255,0.04);
                                     border-radius: 1px 1px 0 0;
                                 "
                             },
                             // 活跃频谱条
                             div {
                                 style: format!(
                                     "position: absolute;
                                     bottom: 0;
                                     left: 0;
                                     right: 0;
                                     height: {}px;
                                     background: linear-gradient(to top, rgba({},{},{},0.95), rgba({},{},{},0.85));
                                     border-radius: 1px 1px 0 0;
                                     transition: height 0.12s ease-out;
                                     box-shadow: 0 0 6px rgba({},{},{},0.3), 0 -2px 10px rgba({},{},{},0.15);
                                     ",
                                     bar_height,
                                     r, g, b,
                                     r2, g2, b2,
                                     r, g, b,
                                     r2, g2, b2,
                                 )
                             }
                         }
                     }
                 }
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
      let select_song = move |song: Song| {
          current_song.set(Some(song.clone()));
          match player().play(&song.file_path) {
              Ok(_) => {
                  is_playing.set(true);
                  // Fill queue with songs after current for playback modes
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
         TitleBar { theme: theme.clone(), on_import: on_import.clone() }
         MainPanel {
             player: player.clone(),
             current_song: current_song.clone(),
             is_playing: is_playing.clone(),
             progress: progress.clone(),
             progress_secs: progress_secs.clone(),
             volume: volume.clone(),
             lyrics: lyrics.clone(),
             db: db.clone(),
             on_song_select: Callback::new(select_song),
             refresh_songs: refresh_songs.clone(),
         }
         VUMeterOverlay { spectrum: spectrum.clone() }
         PlayerBar {
             player: player.clone(),
             current_song: current_song.clone(),
             is_playing: is_playing.clone(),
             progress: progress.clone(),
             volume: volume.clone(),
             playback_mode: playback_mode.clone(),
             is_seeking: is_seeking.clone(),
             output_devices: output_devices.clone(),
         }
     }
}
