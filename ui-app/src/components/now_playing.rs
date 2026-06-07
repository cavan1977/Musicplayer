// Now Playing Component - Song Info Display
use dioxus::prelude::*;
use crate::theme::{use_theme, UiStyle};

#[derive(Props, Clone)]
pub struct NowPlayingProps {
    pub title: String,
    pub artist: String,
    // Optional: album, duration, etc.
}

pub fn NowPlaying(cx: Scope<NowPlayingProps>) -> Element {
    let theme_manager = use_theme(&cx);
    let theme = theme_manager.current();
    let style_sheet = theme_manager.style_sheet.read();

    let (title_style, artist_style) = match theme {
        UiStyle::AquaGlass => (
            format!("font-size: 18px; font-weight: 600; margin-bottom: 4px; color: {};", style_sheet.text_primary),
            format!("font-size: 13px; color: {};", style_sheet.text_secondary),
        ),
        UiStyle::SonicFlux => (
            format!("font-size: 16px; font-weight: 700; letter-spacing: 1px; color: {}; text-transform: uppercase;", style_sheet.text_primary),
            format!("font-size: 12px; color: {}; letter-spacing: 1px;", style_sheet.accent),
        ),
        UiStyle::VintagePro => (
            format!("font-size: 16px; font-weight: 600; margin-bottom: 2px; color: {}; text-shadow: 0 1px 0 rgba(0,0,0,0.5);", style_sheet.text_primary),
            format!("font-size: 13px; color: {}; font-weight: 500;", style_sheet.text_secondary),
        ),
    };

    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                padding: 8px 12px;
                {if let UiStyle::AquaGlass = theme {format!("background: {}; backdrop-filter: {}; border-radius: {};", style_sheet.glass_bg, style_sheet.glass_blur, style_sheet.radius_medium)} else if let UiStyle::VintagePro = theme {format!("background: rgba(0,0,0,0.3); border: 1px solid var(--secondary); border-radius: {};", style_sheet.radius_small)} else {String::new()}
            ",
            div {
                style: "{title_style}",
                "{cx.props.title}"
            },
            div {
                style: "{artist_style}",
                "{cx.props.artist}"
            }
        }
    }
}