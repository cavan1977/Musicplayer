// ui/src/components/song_card.rs
use dioxus::prelude::*;
use crate::AppState;

#[derive(Props)]
pub struct SongCardProps<'a> {
    pub app_state: &'a AppState,
    pub cover_url: String,
    pub title: String,
    pub artist: String,
    pub on_play: EventHandler<'a, ()>,
}

pub fn SongCard<'a>(cx: Scope<'a, SongCardProps<'a>>) -> Element {
    cx.render(rsx! {
        div {
            class: "song-card",
            img {
                class: "cover",
                src: "{cx.props.cover_url}",
                alt: "{cx.props.title} cover"
            }
            div {
                class: "info",
                h3 { "{cx.props.title}" }
                p { "{cx.props.artist}" }
            }
            button {
                class: "play-btn",
                onclick: move |_| cx.props.on_play.call(()),
                "▶"
            }
        }
    })
}