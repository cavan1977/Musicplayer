// ui/src/components/lyrics.rs
use dioxus::prelude::*;

#[derive(Props)]
pub struct LyricsProps<'a> {
    pub lyrics: &'a Vec<LyricLine>,
    pub current_time: f64, // 当前播放时间（秒）
}

pub struct LyricLine {
    pub time_secs: f64,
    pub text: String,
}

pub fn LyricsView<'a>(cx: Scope<'a, LyricsProps<'a>>) -> Element {
    // 计算当前歌词行
    let current_line = cx.props.lyrics
        .iter()
        .position(|line| line.time_secs > cx.props.current_time)
        .unwrap_or(cx.props.lyrics.len());
    
    cx.render(rsx! {
        div {
            class: "lyrics-container",
            cx.props.lyrics.iter().enumerate().map(|(i, line)| {
                let class = if i == current_line {
                    "lyric-line active"
                } else {
                    "lyric-line"
                };
                rsx! {
                    div {
                        key: "{i}",
                        class: "{class}",
                        "{line.text}"
                    }
                }
            })
        }
    })
}