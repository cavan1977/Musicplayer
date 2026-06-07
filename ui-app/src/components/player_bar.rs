use dioxus::prelude::*;
use crate::theme::{use_theme, UiStyle};
use audio::playback::Player;
use db::Database;
use db::Song as DBSong;

// Alias DBSong to Song for consistency
type Song = DBSong;

#[derive(Props, Clone)]
pub struct PlayerBarProps {
    pub player: Signal<Player>,
    pub db: Signal<Database>,
    pub current_song: Signal<Option<Song>>,
    pub is_playing: Signal<bool>,
    pub progress: Signal<f64>,
    pub volume: Signal<f64>,
}

pub fn PlayerBar(cx: Scope<PlayerBarProps>) -> Element {
    let theme_manager = use_theme(&cx);
    let theme = theme_manager.current();
    let style_sheet = theme_manager.style_sheet.read();

    let player_ref = cx.props.player;
    let is_playing = cx.props.is_playing;
    let progress = cx.props.progress;
    let volume = cx.props.volume;

    // Handlers
    let toggle_play = {
        let player = player_ref.clone();
        let is_playing = is_playing.clone();
        move |_| {
            if is_playing() {
                player().pause();
                is_playing.set(false);
            } else {
                player().resume();
                is_playing.set(true);
            }
        }
    };

    let prev_track = move |_| {
        // TODO: previous track in queue
    };

    let next_track = move |_| {
        // TODO: next track in queue
    };

    let seek = {
        let player = player_ref.clone();
        move |pct: f64| {
            let _ = player().seek(pct);
        }
    };

    let set_volume = {
        let player = player_ref.clone();
        move |vol: f64| {
            // TODO: player().set_volume_db(vol);
        }
    };

    // Styles per theme
    let (container_style, controls_style, progress_style, volume_style) = match theme {
        UiStyle::AquaGlass => (
            format!(
                "height: 80px; background: rgba(0,0,0,0.4); backdrop-filter: blur(20px); border-top: 1px solid rgba(255,255,255,0.1); padding: 0 24px; display: flex; align-items: center; gap: 24px;"
            ),
            "display: flex; gap: 12px;".to_string(),
            format!("
                flex: 1;
                -webkit-appearance: none;
                height: 6px;
                border-radius: 3px;
                background: rgba(255,255,255,0.2);
                outline: none;
                {}",
                format!("&::-webkit-slider-thumb {{ -webkit-appearance: none; width: 16px; height: 16px; border-radius: 50%; background: var(--accent); cursor: pointer; box-shadow: 0 0 10px var(--accent); }}")
            ),
            "width: 80px; -webkit-appearance: none; height: 6px; border-radius: 3px; background: rgba(255,255,255,0.2); outline: none;".to_string(),
        ),
        UiStyle::SonicFlux => (
            format!(
                "height: 70px; background: rgba(10,10,15,0.9); border-top: 1px solid var(--primary); box-shadow: 0 0 10px rgba(0,255,255,0.2); padding: 0 24px; display: flex; align-items: center; gap: 20px;"
            ),
            "display: flex; gap: 10px;".to_string(),
            format!("
                flex: 1;
                -webkit-appearance: none;
                height: 8px;
                border-radius: 0;
                background: rgba(0,255,255,0.2);
                outline: none;
                {}",
                format!("&::-webkit-slider-thumb {{ -webkit-appearance: none; width: 12px; height: 12px; background: var(--primary); cursor: pointer; box-shadow: 0 0 8px var(--primary); }}")
            ),
            "width: 80px; -webkit-appearance: none; height: 8px; border-radius: 0; background: rgba(0,255,255,0.3); outline: none;".to_string(),
        ),
        UiStyle::VintagePro => (
            format!(
                "height: 75px; background: linear-gradient(180deg, #2d1f15 0%, #1a0f0a 100%); border-top: 2px solid var(--secondary); padding: 0 20px; display: flex; align-items: center; gap: 16px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.5);"
            ),
            "display: flex; gap: 8px;".to_string(),
            format!("
                flex: 1;
                -webkit-appearance: none;
                height: 12px;
                border-radius: 6px;
                background: #0a0705;
                border: 1px solid var(--secondary);
                outline: none;
                {}",
                format!("&::-webkit-slider-thumb {{ -webkit-appearance: none; width: 18px; height: 18px; border-radius: 50%; background: radial-gradient(circle, var(--accent) 0%, var(--primary) 100%); cursor: pointer; box-shadow: 0 2px 4px rgba(0,0,0,0.5); }}")
            ),
            "width: 80px; -webkit-appearance: none; height: 12px; border-radius: 6px; background: #0a0705; border: 1px solid var(--secondary); outline: none;".to_string(),
        ),
    };

    // Cover art (small)
    let cover_style = match theme {
        UiStyle::AquaGlass => "width: 56px; height: 56px; border-radius: 12px; background: rgba(255,255,255,0.1); display: flex; align-items: center; justify-content: center; font-size: 24px; flex-shrink: 0;".to_string(),
        UiStyle::SonicFlux => "width: 56px; height: 56px; border: 2px solid var(--primary); display: flex; align-items: center; justify-content: center; font-size: 24px; box-shadow: 0 0 8px var(--primary); transform: rotate(45deg); flex-shrink: 0;".to_string(),
        UiStyle::VintagePro => "width: 56px; height: 56px; border: 2px solid var(--secondary); border-radius: 2px; background: rgba(0,0,0,0.3); display: flex; align-items: center; justify-content: center; font-size: 24px; flex-shrink: 0; box-shadow: inset 0 1px 0 rgba(255,255,255,0.1);".to_string(),
    };

    // Button styles
    let (play_btn_style, skip_btn_style) = match theme {
        UiStyle::AquaGlass => (
            "width: 44px; height: 44px; border-radius: 22px; border: none; background: var(--accent); color: white; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 16px; box-shadow: 0 4px 12px rgba(255,107,107,0.4); transition: transform 0.1s;".to_string(),
            "width: 36px; height: 36px; border-radius: 18px; border: none; background: rgba(255,255,255,0.1); color: white; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 14px;".to_string(),
        ),
        UiStyle::SonicFlux => (
            "width: 40px; height: 40px; border: 1px solid var(--primary); border-radius: 0; background: transparent; color: var(--primary); cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 16px; box-shadow: 0 0 8px var(--primary);".to_string(),
            "width: 32px; height: 32px; border: 1px solid var(--secondary); border-radius: 0; background: transparent; color: var(--secondary); cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 12px;".to_string(),
        ),
        UiStyle::VintagePro => (
            "width: 42px; height: 42px; border: 1px solid var(--secondary); border-radius: 4px; background: linear-gradient(180deg, var(--primary) 0%, var(--secondary) 100%); color: white; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 16px; box-shadow: inset 0 1px 0 rgba(255,255,255,0.3), 0 2px 4px rgba(0,0,0,0.4);".to_string(),
            "width: 34px; height: 34px; border: 1px solid var(--secondary); border-radius: 4px; background: linear-gradient(180deg, #5a4510 0%, #3d2b0a 100%); color: var(--text-primary); cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 14px; box-shadow: inset 0 1px 0 rgba(255,255,255,0.2);".to_string(),
        ),
    };

    rsx! {
        div {
            style: container_style,
            
            // Album cover (small)
            if let Some(song) = cx.props.current_song.as_ref() {
                div {
                    style: "{cover_style}",
                    img {
                        src: song.cover_url.as_deref().unwrap_or("https://via.placeholder.com/56"),
                        style: "width: 100%; height: 100%; object-fit: cover; border-radius: inherit;",
                        alt: "Cover"
                    }
                }
            } else {
                div {
                    style: "{cover_style}",
                    "🎵"
                }
            }
            
            // Track info
            div {
                style: "flex: 0 0 180px; display: flex; flex-direction: column; justify-content: center;",
                if let Some(song) = cx.props.current_song.as_ref() {
                    div {
                        style: "font-weight: 600; margin-bottom: 4px; color: var(--text-primary); font-size: 14px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{song.title}"
                    }
                    div {
                        style: "font-size: 12px; color: var(--text-secondary);",
                        "{song.artist}"
                    }
                } else {
                    div {
                        style: "color: var(--text-secondary); font-size: 14px;",
                        "未选择曲目"
                    }
                }
            }
            
            // Playback controls
            div {
                style: controls_style,
                button {
                    onclick: prev_track,
                    style: "{skip_btn_style}",
                    "⏮"
                }
                button {
                    onclick: toggle_play,
                    style: "{play_btn_style}",
                    if is_playing() { "⏸" } else { "▶" }
                }
                button {
                    onclick: next_track,
                    style: "{skip_btn_style}",
                    "⏭"
                }
            }
            
            // Progress bar
            div {
                style: "flex: 1; display: flex; align-items: center; gap: 12px; font-size: 12px; color: var(--text-secondary);",
                span { "{format_time(progress() * 3.0)}" } // dummy: need duration
                input {
                    r#type: "range",
                    min: "0",
                    max: "100",
                    value: "{progress()}",
                    oninput: move |e| {
                        if let Ok(val) = e.data.value().parse::<f64>() {
                            seek(val);
                        }
                    },
                    style: progress_style
                }
                span { "--:--" } // total duration
            }
            
            // Volume control
            div {
                style: "display: flex; align-items: center; gap: 8px;",
                button {
                    style: "background: none; border: none; color: var(--text-primary); cursor: pointer; font-size: 14px;",
                    if volume() > 0.0 { "🔊" } else { "🔈" }
                }
                input {
                    r#type: "range",
                    min: "0",
                    max: "100",
                    value: "{volume() * 100.0}",
                    oninput: move |e| {
                        if let Ok(val) = e.data.value().parse::<f64>() {
                            let vol = val / 100.0;
                            set_volume(vol);
                            volume.set(vol);
                        }
                    },
                    style: volume_style
                }
            }
        }
    }
}