//! Sonic Flux Theme - 声波流风格
//! 仪表盘式布局：顶部信息栏 + 下方左(频谱+曲目) | 右(歌词)
//! 赛博朋克 HUD、锐利边角、霓虹光晕、等宽字体、无边框、全中文

use dioxus::prelude::*;
use dioxus_desktop::use_window;
use db::Database;
use audio::Player;
use audio::backend::PlaybackState;
use audio::PlaybackMode;
use crate::lyrics_parser::Lyrics;
use crate::{Song, format_time, UiStyle, ThemeManager};

// ==================== HUDHeader ====================
#[derive(Props, PartialEq, Clone)]
struct HUDHeaderProps {
    pub theme: Signal<ThemeManager>,
    pub on_import: Callback<()>,
}

#[component]
fn HUDHeader(props: HUDHeaderProps) -> Element {
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
                height: 56px;
                background: rgba(5,5,16,0.95);
                border-bottom: 1px solid rgba(0,240,255,0.5);
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 0 20px;
                box-shadow: 0 2px 16px rgba(0,240,255,0.15);
                user-select: none;
                -webkit-app-region: drag;
                position: relative;
                overflow: hidden;
            ",
            div {
                style: "
                    position: absolute;
                    top: 0; left: 0; right: 0; bottom: 0;
                    background: repeating-linear-gradient(
                        0deg,
                        transparent,
                        transparent 2px,
                        rgba(0,240,255,0.015) 2px,
                        rgba(0,240,255,0.015) 4px
                    );
                    pointer-events: none;
                "
            }
            div {
                style: "display: flex; align-items: center; gap: 10px; -webkit-app-region: no-drag; z-index: 1;",
                div {
                    style: "
                        width: 32px; height: 32px;
                        border: 1px solid var(--color-primary);
                        display: flex; align-items: center; justify-content: center;
                        font-size: 14px;
                        box-shadow: 0 0 8px rgba(0,240,255,0.3), inset 0 0 4px rgba(0,240,255,0.1);
                    ",
                    "◈"
                }
                span {
                    style: "
                        font-family: var(--font-family);
                        font-size: 16px;
                        font-weight: 700;
                        color: var(--color-primary);
                        text-shadow: 0 0 10px var(--color-primary);
                        letter-spacing: 2px;
                    ",
                    "SONIC//FLUX"
                }
            }
             div {
                 style: "display: flex; align-items: center; gap: 6px; -webkit-app-region: no-drag; z-index: 1;",
                 ThemeButton { text: "水琉璃".to_string(), active: false, on_click: Callback::new(move |_evt: MouseEvent| { switch(UiStyle::AquaGlass); }) }
                 ThemeButton { text: "声波流".to_string(), active: true, on_click: Callback::new(move |_evt: MouseEvent| { switch(UiStyle::SonicFlux); }) }
                 ThemeButton { text: "经典专业".to_string(), active: false, on_click: Callback::new(move |_evt: MouseEvent| { switch(UiStyle::VintagePro); }) }
                 div {
                     style: "width: 1px; height: 20px; background: rgba(0,240,255,0.3); margin: 0 4px;"
                 }
                 button {
                     onclick: move |_| on_import.call(()),
                     style: "
                         padding: 4px 12px;
                         border: 1px solid var(--color-accent);
                         background: rgba(0,255,157,0.08);
                         color: var(--color-accent);
                         cursor: pointer;
                         font-size: 11px;
                         font-family: var(--font-family);
                         text-shadow: 0 0 4px var(--color-accent);
                         box-shadow: 0 0 6px rgba(0,255,157,0.2);
                     ",
                     "⟨导入⟩"
                 }
                 button {
                     onclick: move |_| window_min.set_minimized(true),
                     style: "
                         width: 28px;
                         height: 28px;
                         border: 1px solid rgba(0,240,255,0.3);
                         background: transparent;
                         color: var(--color-primary);
                         cursor: pointer;
                         font-size: 14px;
                         font-family: var(--font-family);
                     ",
                     "−"
                 }
                 button {
                     onclick: move |_| window_close.close(),
                     style: "
                         width: 28px;
                         height: 28px;
                         border: none;
                         background: rgba(255,50,50,0.8);
                         color: white;
                         cursor: pointer;
                         font-size: 12px;
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
                "padding: 8px 16px; border: 1px solid {}; border-radius: 0; color: {}; font-family: var(--font-family); font-size: 13px; cursor: pointer; transition: all 0.2s; background: {}; box-shadow: {}; text-shadow: {};",
                if active { "var(--color-accent)" } else { "rgba(0,240,255,0.3)" },
                if active { "var(--color-accent)" } else { "rgba(0,240,255,0.7)" },
                if active { "rgba(0,240,255,0.2)" } else { "transparent" },
                if active { "inset 0 0 6px var(--color-accent)" } else { "none" },
                if active { "0 0 6px var(--color-accent)" } else { "none" }
            ),
            "{props.text}"
        }
    }
}

// ==================== NowPlayingStrip (顶部曲目信息条) ====================
#[derive(Props, PartialEq, Clone)]
struct NowPlayingStripProps {
    pub current_song: Signal<Option<Song>>,
}

#[component]
fn NowPlayingStrip(props: NowPlayingStripProps) -> Element {
    let current_song = props.current_song;

    rsx! {
        div {
            style: "
                height: 72px;
                background: rgba(0,0,0,0.6);
                border-bottom: 1px solid rgba(0,240,255,0.3);
                display: flex;
                align-items: center;
                padding: 0 20px;
                gap: 16px;
                flex-shrink: 0;
                position: relative;
                overflow: hidden;
            ",
            // 扫描线效果
            div {
                style: "
                    position: absolute;
                    top: 0; left: 0; right: 0; bottom: 0;
                    background: repeating-linear-gradient(
                        0deg,
                        transparent,
                        transparent 2px,
                        rgba(0,240,255,0.01) 2px,
                        rgba(0,240,255,0.01) 4px
                    );
                    pointer-events: none;
                "
            }
            // 矩形封面
            if let Some(song) = &*current_song.read() {
                if let Some(cover_url) = song.cover_url.as_ref() {
                    img {
                        src: cover_url,
                        style: "width: 52px; height: 52px; object-fit: cover; border: 1px solid rgba(0,240,255,0.4); box-shadow: 0 0 12px rgba(0,240,255,0.2); flex-shrink: 0;"
                    }
                } else {
                    div {
                        style: "
                            width: 52px; height: 52px;
                            background: rgba(0,240,255,0.05);
                            border: 1px solid rgba(0,240,255,0.3);
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            font-size: 22px;
                            flex-shrink: 0;
                        ",
                        "♫"
                    }
                }
            } else {
                div {
                    style: "
                        width: 52px; height: 52px;
                        background: rgba(0,240,255,0.03);
                        border: 1px solid rgba(0,240,255,0.15);
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 22px; color: rgba(0,240,255,0.2);
                        flex-shrink: 0;
                    ",
                    "◈"
                }
            }
            // 曲目信息
            div {
                style: "flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px;",
                div {
                    style: "font-family: var(--font-family); font-size: 9px; color: rgba(0,240,255,0.5); letter-spacing: 2px; text-transform: uppercase;",
                    "▸ NOW PLAYING ◂"
                }
                if let Some(song) = &*current_song.read() {
                    div {
                        style: "font-size: 16px; font-weight: 700; color: var(--color-primary); text-shadow: 0 0 8px var(--color-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{song.title}"
                    }
                    div {
                        style: "font-size: 12px; color: var(--text-secondary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{song.artist}"
                    }
                } else {
                    div {
                        style: "font-size: 14px; color: rgba(0,240,255,0.3);",
                        "[ 等待输入 ]"
                    }
                }
            }
            // 状态指示灯
            div {
                style: "display: flex; gap: 6px; flex-shrink: 0;",
                div { style: "width: 6px; height: 6px; border-radius: 50%; background: rgba(0,255,157,0.7); box-shadow: 0 0 6px rgba(0,255,157,0.5);" }
                div { style: "width: 6px; height: 6px; border-radius: 50%; background: rgba(0,240,255,0.5); box-shadow: 0 0 4px rgba(0,240,255,0.3);" }
                div { style: "width: 6px; height: 6px; border-radius: 50%; background: rgba(255,0,255,0.4); box-shadow: 0 0 4px rgba(255,0,255,0.3);" }
            }
        }
    }
}

// ==================== LyricsOverlay ====================
#[derive(Props, PartialEq, Clone)]
struct LyricsOverlayProps {
    pub lyrics: Signal<Option<Lyrics>>,
    pub progress_secs: Signal<f64>,
}

#[component]
fn LyricsOverlay(props: LyricsOverlayProps) -> Element {
    let lyrics = props.lyrics;
    let progress_secs = props.progress_secs;

    let elapsed_secs = *progress_secs.read();

    let active_data: Option<(Vec<(f64, String)>, usize)> = if let Some(lrc) = &*lyrics.read() {
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
                            "var c=document.getElementById('lyrics-container-sonic'); \
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
            id: "lyrics-container-sonic",
            style: "
                flex: 1;
                background: rgba(0,0,0,0.5);
                border: 1px solid rgba(0,240,255,0.25);
                padding: 16px;
                overflow-y: auto;
                mask-image: linear-gradient(transparent 0%, black 8%, black 92%, transparent 100%);
                position: relative;
            ",
            div {
                style: "
                    position: absolute;
                    top: 0; left: 0; right: 0; height: 1px;
                    background: linear-gradient(90deg, transparent, var(--color-primary), var(--color-accent), transparent);
                "
            }
            div {
                style: "
                    font-family: var(--font-family);
                    font-size: 9px;
                    color: rgba(0,240,255,0.4);
                    letter-spacing: 2px;
                    margin-bottom: 12px;
                ",
                "▸ LYRICS DATA STREAM ◂"
            }
            if let Some((lines, active_idx_val)) = active_data {
                for (i, (_, text)) in lines.iter().enumerate() {
                    div {
                        key: "{i}",
                        id: format!("lyric-line-{}", i),
                        style: format!(
                            "line-height: 2.0; padding: 6px 12px; color: {}; font-family: var(--font-family); font-size: {}; transition: all 0.3s ease-out; opacity: {}; font-weight: {}; text-shadow: {}; border-left: {};",
                            if i == active_idx_val { "var(--color-primary)" } else { "var(--text-secondary)" },
                            if i == active_idx_val { "16px" } else { "13px" },
                            if i == active_idx_val { "1" } else {
                                let dist = if active_idx_val > i { active_idx_val - i } else { i - active_idx_val };
                                if dist <= 1 { "0.6" } else if dist <= 3 { "0.3" } else { "0.15" }
                            },
                            if i == active_idx_val { "700" } else { "400" },
                            if i == active_idx_val { "0 0 10px var(--color-primary)" } else { "none" },
                            if i == active_idx_val { "2px solid var(--color-primary)" } else { "2px solid transparent" }
                        ),
                        "{text}"
                    }
                }
            } else {
                div {
                    style: "
                        color: rgba(0,240,255,0.2);
                        font-size: 13px;
                        text-align: center;
                        padding: 40px 20px;
                        font-family: var(--font-family);
                    ",
                    "[ 无歌词数据 ]"
                }
            }
        }
    }
}

// ==================== SongMatrixList ====================
#[derive(Props, PartialEq, Clone)]
struct SongMatrixListProps {
    pub db: Signal<Database>,
    pub on_song_select: Callback<Song>,
    pub current_song: Signal<Option<Song>>,
    pub refresh_songs: Signal<u32>,
}

#[component]
fn SongMatrixList(props: SongMatrixListProps) -> Element {
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
                 flex: 1;
                 background: rgba(0,0,0,0.4);
                 border: 1px solid rgba(0,240,255,0.2);
                 padding: 16px;
                 overflow-y: auto;
                 position: relative;
             ",
             div {
                 style: "
                     position: absolute;
                     top: 0; left: 0; right: 0; height: 1px;
                     background: linear-gradient(90deg, transparent, rgba(0,255,157,0.4), transparent);
                 "
             }
             h3 {
                 style: "
                     font-size: 11px;
                     font-weight: 700;
                     color: var(--color-accent);
                     margin: 0 0 12px 0;
                     padding-bottom: 8px;
                     border-bottom: 1px dashed rgba(0,240,255,0.2);
                     letter-spacing: 2px;
                     text-transform: uppercase;
                 ",
                 "▸ TRACK MATRIX ◂"
             }
             if songs_guard.is_empty() {
                 div {
                     style: "
                         color: rgba(0,240,255,0.2);
                         font-size: 12px;
                         padding: 30px;
                         text-align: center;
                         font-family: var(--font-family);
                     ",
                     "[ 无数据 ]"
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
                                     "padding: 8px 12px; margin-bottom: 2px; cursor: pointer; transition: all 0.15s; border-left: {}; background: {};",
                                     if is_current { "2px solid var(--color-primary)" } else { "2px solid transparent" },
                                     if is_current { "rgba(0,240,255,0.1)" } else { "rgba(255,255,255,0.02)" }
                                 ),
                                 div {
                                     style: format!(
                                         "font-weight: {}; color: {}; font-size: 13px; margin-bottom: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; text-shadow: {};",
                                         if is_current { "700" } else { "500" },
                                         if is_current { "var(--color-primary)" } else { "var(--text-primary)" },
                                         if is_current { "0 0 6px var(--color-primary)" } else { "none" }
                                     ),
                                     "{&song.title}"
                                 },
                                 div {
                                     style: "font-size: 11px; color: rgba(255,255,255,0.35); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
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

 // ==================== VUDisplay ====================
 #[derive(Props, PartialEq, Clone)]
 struct VUDisplayProps {
     pub spectrum: Signal<[f32; 64]>,
 }
 
 #[component]
 fn VUDisplay(props: VUDisplayProps) -> Element {
let specdata = props.spectrum.read();
      rsx! {
         div {
             style: "
                 position: relative;
                 height: 140px;
                 background: rgba(0,0,0,0.85);
                 border-top: 2px solid rgba(0,255,255,0.5);
                 padding: 8px 12px 6px 12px;
                 display: flex;
                 flex-direction: column;
                 gap: 0;
                 overflow: hidden;
                 box-shadow: inset 0 2px 20px rgba(0,255,255,0.1);
             ",
             // Header label row
             div {
                 style: "
                     display: flex;
                     justify-content: space-between;
                     align-items: center;
                     margin-bottom: 6px;
                 ",
                 div {
                     style: "
                         font-family: 'JetBrains Mono', 'Courier New', monospace;
                         font-size: 10px;
                         font-weight: 600;
                         color: rgba(0,255,255,0.6);
                         letter-spacing: 2px;
                         text-transform: uppercase;
                         text-shadow: 0 0 8px rgba(0,255,255,0.8);
                     ",
                     "▸ SPECTRUM ANALYZER ◂"
                 }
                 // dB markings at right
                 div {
                     style: "
                         display: flex;
                         flex-direction: column;
                         justify-content: space-between;
                         height: 100px;
                         align-items: flex-end;
                         font-family: 'JetBrains Mono', 'Courier New', monospace;
                         font-size: 8px;
                         color: rgba(0,255,255,0.4);
                     ",
                     div { "0 dB" }
                     div { "-20" }
                     div { "-40" }
                     div { "-60" }
                 }
             }
             // Spectrum bars container with grid overlay
             div {
                 style: "
                     position: relative;
                     flex: 1;
                     display: flex;
                     align-items: flex-end;
                     gap: 1px;
                 ",
                 // Horizontal grid lines (HUD aesthetic)
                 div {
                     style: "
                         position: absolute;
                         left: 0; right: 50px;
                         height: 1px;
                         background: rgba(0,255,255,0.15);
                         pointer-events: none;
                     "
                 }
                 div {
                     style: "
                         position: absolute;
                         left: 0; right: 50px;
                         bottom: 25%;
                         height: 1px;
                         background: rgba(0,255,255,0.15);
                         pointer-events: none;
                     "
                 }
                 div {
                     style: "
                         position: absolute;
                         left: 0; right: 50px;
                         bottom: 50%;
                         height: 1px;
                         background: rgba(0,255,255,0.15);
                         pointer-events: none;
                     "
                 }
                 div {
                     style: "
                         position: absolute;
                         left: 0; right: 50px;
                         bottom: 75%;
                         height: 1px;
                         background: rgba(0,255,255,0.15);
                         pointer-events: none;
                     "
                 }
// 64 Spectrum bars
                  for i in 0..64 {
                      {
                          let intensity = specdata.get(i).copied().unwrap_or(0.0);
                          let bar_h = (if intensity > 0.0 { 4.0 + intensity * 96.0 } else { 4.0 }) as i32;
                          let glow = if intensity > 0.3 { 0.8 } else if intensity > 0.1 { 0.5 } else { 0.2 };
                          let g1: f64 = if intensity > 0.5 { 0.8 } else { 0.6 };
                          let g2: f64 = if intensity > 0.7 { 0.9 } else { 0.6 };
                          let g3: f64 = if intensity > 0.85 { 1.0 } else { 0.3 };
                          let g4: i32 = if intensity > 0.5 { 255 } else { 180 };
                          rsx! {
                              div {
                                  key: "{i}",
                                  style: format!(
                                      "flex: 1; min-width: 2px; max-width: 8px; height: {}px; background: linear-gradient(to top, rgba(0,255,255,{}), rgba(180,0,255,{}), rgba(255,255,255,{})); box-shadow: 0 0 6px rgba(0,{},{}); transition: height 0.12s ease-out;",
                                      bar_h, g1, g2, g3, g4, glow
                                  )
                              }
                          }
                      }
                  }
             }
             // Bottom scan line accent
             div {
                 style: "
                     position: absolute;
                     bottom: 0; left: 0; right: 0; height: 1px;
                     background: linear-gradient(90deg, transparent, rgba(0,255,255,0.8), rgba(255,0,255,0.6), transparent);
                 "
             }
             // Corner HUD brackets
             div {
                 style: "
                     position: absolute;
                     top: 0; left: 0;
                     width: 12px; height: 12px;
                     border-top: 2px solid rgba(0,255,255,0.7);
                     border-left: 2px solid rgba(0,255,255,0.7);
                 "
             }
             div {
                 style: "
                     position: absolute;
                     top: 0; right: 60px;
                     width: 12px; height: 12px;
                     border-top: 2px solid rgba(255,0,255,0.7);
                     border-right: 2px solid rgba(255,0,255,0.7);
                 "
             }
             div {
                 style: "
                     position: absolute;
                     bottom: 0; left: 0;
                     width: 12px; height: 12px;
                     border-bottom: 2px solid rgba(0,255,255,0.5);
                     border-left: 2px solid rgba(0,255,255,0.5);
                 "
             }
             div {
                 style: "
                     position: absolute;
                     bottom: 0; right: 60px;
                     width: 12px; height: 12px;
                     border-bottom: 2px solid rgba(255,0,255,0.5);
                     border-right: 2px solid rgba(255,0,255,0.5);
                 "
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
     pub spectrum: Signal<[f32; 64]>,
     pub refresh_songs: Signal<u32>,
 }

 #[component]
 fn MainPanel(props: MainPanelProps) -> Element {
     rsx! {
         div {
             style: "
                 flex: 1;
                 display: flex;
                 gap: 0;
                 min-height: 0;
                 overflow: hidden;
             ",
             // 左半区：频谱 + 曲目列表 (垂直堆叠)
             div {
                 style: "flex: 1; display: flex; flex-direction: column; min-width: 0; overflow: hidden;",
                 VUDisplay { spectrum: props.spectrum.clone() }
                 SongMatrixList {
                     db: props.db.clone(),
                     on_song_select: props.on_song_select.clone(),
                     current_song: props.current_song.clone(),
                     refresh_songs: props.refresh_songs.clone(),
                 }
             }
             // 右半区：歌词（大区域）
             div {
                 style: "flex: 1; display: flex; flex-direction: column; min-width: 0; overflow: hidden; padding: 12px 12px 12px 0;",
                 LyricsOverlay {
                     lyrics: props.lyrics.clone(),
                     progress_secs: props.progress_secs.clone(),
                 }
             }
         }
     }
 }

// ==================== ControlConsole ====================
#[derive(Props, PartialEq, Clone)]
struct ControlConsoleProps {
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
fn ControlConsole(props: ControlConsoleProps) -> Element {
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
    let time_display = format!("{} / {}", format_time(elapsed), format_time(total_secs));
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
                height: 84px;
                background: rgba(5,5,16,0.95);
                border-top: 1px solid rgba(0,240,255,0.4);
                display: flex;
                align-items: center;
                padding: 0 20px;
                gap: 16px;
                box-shadow: 0 -2px 16px rgba(0,240,255,0.15);
                position: relative;
                flex-shrink: 0;
            ",
            div {
                style: "
                    position: absolute;
                    top: 0; left: 20px; right: 20px; height: 1px;
                    background: linear-gradient(90deg, transparent, var(--color-accent), transparent);
                    opacity: 0.5;
                "
            },
            if let Some(song) = current_song.read().as_ref() {
                if let Some(cover_url) = song.cover_url.as_ref() {
                    img {
                        src: cover_url,
                        style: "width: 42px; height: 42px; border-radius: 50%; object-fit: cover; flex-shrink: 0; border: 1px solid var(--color-primary); box-shadow: 0 0 8px rgba(0,240,255,0.3);"
                    }
                } else {
                    div {
                        style: "
                            width: 42px;
                            height: 42px;
                            border-radius: 50%;
                            background: radial-gradient(circle, #1a1a1a 0%, #0a0a0a 100%);
                            border: 1px solid var(--color-primary);
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            font-size: 16px;
                            flex-shrink: 0;
                            box-shadow: 0 0 8px rgba(0,240,255,0.2);
                        ",
                        "♫"
                    }
                }
            } else {
                div {
                    style: "
                        width: 42px;
                        height: 42px;
                        border-radius: 50%;
                        background: rgba(0,240,255,0.05);
                        border: 1px solid rgba(0,240,255,0.2);
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 16px;
                        flex-shrink: 0;
                    ",
                    "◈"
                }
            }

            div {
                style: "flex: 1; min-width: 0; display: flex; flex-direction: column; justify-content: center;",
                if let Some(song) = &*current_song.read() {
                    div {
                        style: "
                            font-size: 14px;
                            font-weight: 700;
                            color: var(--text-primary);
                            white-space: nowrap;
                            overflow: hidden;
                            text-overflow: ellipsis;
                            text-shadow: 0 0 4px var(--color-primary);
                        ",
                        "{song.title}"
                    }
                    div {
                        style: "
                            font-size: 12px;
                            color: var(--text-secondary);
                            white-space: nowrap;
                            overflow: hidden;
                            text-overflow: ellipsis;
                        ",
                        "{song.artist}"
                    }
                } else {
                    div { style: "font-size: 14px; font-weight: 700; color: rgba(0,240,255,0.3);", "[ 等待输入 ]" }
                }
            }

            div {
                style: "flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px;",
                 div {
                     style: "display: flex; align-items: center; gap: 12px;",
                     button {
                         onclick: toggle_play,
                         style: "
                             width: 42px;
                             height: 42px;
                             border-radius: 50%;
                             background: rgba(0,240,255,0.15);
                             border: 1px solid var(--color-primary);
                             color: var(--color-primary);
                             font-size: 16px;
                             cursor: pointer;
                             box-shadow: 0 0 12px rgba(0,240,255,0.3);
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
                              background: rgba(0,240,255,0.1);
                              border: 1px solid var(--color-primary);
                              color: var(--color-primary);
                              font-size: 12px;
                              cursor: pointer;
                              display: flex;
                              align-items: center;
                              justify-content: center;
                              box-shadow: 0 0 6px rgba(0,240,255,0.2);
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
                         style: "flex: 1; display: flex; flex-direction: column; gap: 4px;",
                        div {
                            style: "
                                font-size: 11px;
                                color: var(--color-accent);
                                font-variant-numeric: tabular-nums;
                                text-align: right;
                                font-family: var(--font-family);
                            ",
                            "{time_display}"
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
                                width: 100%;
                                height: 6px;
                                border-radius: 3px;
                                background: rgba(0,240,255,0.25);
                                outline: none;
                                box-shadow: 0 0 4px rgba(0,240,255,0.3);
                            "
                        }
                    }
                }

                div {
                    style: "display: flex; align-items: center; gap: 6px; padding-left: 54px;",
                    span {
                        style: "font-size: 11px; color: rgba(0,240,255,0.5); font-family: var(--font-family);",
                        "VOL"
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
                            background: rgba(0,240,255,0.25);
                            outline: none;
                            box-shadow: 0 0 4px rgba(0,240,255,0.3);
                        "
                    }
                    span {
                        style: "font-size: 11px; color: var(--text-secondary); width: 32px; text-align: right; font-family: var(--font-family);",
                        {format!("{:.0}%", volume_val * 100.0)}
                    }
                    // 设备选择
                    select {
                        onchange: move |evt: Event<FormData>| {
                            let val = evt.value();
                            let device_name = if val == "__default__" { None } else { Some(val) };
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
                            font-size: 10px;
                            padding: 2px 4px;
                            border: 1px solid var(--color-primary);
                            background: rgba(0,240,255,0.08);
                            color: var(--color-primary);
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

// ==================== App ====================
#[derive(Props, PartialEq, Clone)]
pub struct AppProps {
    pub theme: Signal<ThemeManager>,
    pub player: Signal<Player>,
    pub db: Signal<Database>,
    pub current_song: Signal<Option<Song>>,
    pub is_playing: Signal<bool>,
    pub progress: Signal<f64>,
    pub progress_secs: Signal<f64>,
    pub volume: Signal<f64>,
    pub lyrics: Signal<Option<Lyrics>>,
    pub on_import: Callback<()>,
    pub spectrum: Signal<[f32; 64]>,
    pub refresh_songs: Signal<u32>,
    pub playback_mode: Signal<PlaybackMode>,
    pub is_seeking: Signal<bool>,
    pub output_devices: Signal<Vec<String>>,
}

#[component]
pub fn App(props: AppProps) -> Element {
    let theme = props.theme;
    let player = props.player;
    let db = props.db;
    let mut current_song = props.current_song;
    let mut is_playing = props.is_playing;
    let progress = props.progress;
    let progress_secs = props.progress_secs;
    let volume = props.volume;
    let mut lyrics = props.lyrics;
    let on_import = props.on_import;
    let spectrum = props.spectrum;
    let refresh_songs = props.refresh_songs;
    let playback_mode = props.playback_mode;
    let is_seeking = props.is_seeking;

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
         HUDHeader { theme: theme.clone(), on_import: on_import.clone() }
         NowPlayingStrip { current_song: current_song.clone() }
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
             spectrum: spectrum.clone(),
             refresh_songs: refresh_songs.clone(),
         }
         ControlConsole {
             player: player.clone(),
             current_song: current_song.clone(),
             is_playing: is_playing.clone(),
             progress: progress.clone(),
             volume: volume.clone(),
             playback_mode: playback_mode.clone(),
             is_seeking: is_seeking.clone(),
             output_devices: props.output_devices.clone(),
         }
     }
}
