// ui/src/components/player.rs
use dioxus::prelude::*;
use crate::{AppState, components::lyrics::LyricsView};

#[derive(Props, PartialEq)]
pub struct PlayerControlsProps<'a> {
    app_state: &'a AppState,
    current_song: Option<Song>,
    current_time: f64,
    lyrics: Vec<LyricLine>,
}

#[derive(Clone, PartialEq)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub cover_url: String,
    pub duration_secs: f64,
}

pub fn PlayerControls<'a>(cx: Scope<'a, PlayerControlsProps<'a>>) -> Element {
    let app_state = cx.props.app_state;
    let playback = app_state.playback.lock().unwrap();
    
    cx.render(rsx! {
        div {
            class: "player-layout",
            // 封面和歌曲信息
            if let Some(song) = &cx.props.current_song {
                rsx! {
                    div {
                        class: "song-info",
                        img {
                            class: "cover",
                            src: "{song.cover_url}"
                        }
                        h3 { "{song.title}" }
                        p { "{song.artist}" }
                    }
                }
            }
            
            // 播放控制
            div {
                class: "player-controls",
                button {
                    onclick: move |_| { playback.play().unwrap(); },
                    "▶"
                }
                button {
                    onclick: move |_| { playback.pause().unwrap(); },
                    "⏸"
                }
                input {
                    r#type: "range",
                    min: "0",
                    max: "{cx.props.current_song.as_ref().map_or(100.0, |s| s.duration_secs)}",
                    value: "{cx.props.current_time}",
                    oninput: move |evt| {
                        // TODO：seek 实现
                    }
                }
            }
            
            // 歌词滚动
            rsx! { LyricsView { lyrics: &cx.props.lyrics, current_time: cx.props.current_time } }
        }
    })
}