// Album Cover Component - 3 Distinct Styles
use dioxus::prelude::*;
use crate::theme::{use_theme, UiStyle};

#[derive(Props, Clone)]
pub struct AlbumCoverProps {
    pub cover_url: Option<String>,
    pub is_playing: Signal<bool>,
    pub size: Option<f32>, // pixels
}

pub fn AlbumCover(cx: Scope<AlbumCoverProps>) -> Element {
    let theme_manager = use_theme(&cx);
    let theme = theme_manager.current();
    let style_sheet = theme_manager.style_sheet.read();
    let size = cx.props.size.unwrap_or(200.0) as f32;

    // Default placeholder gradient based on theme
    let (bg_gradient, shadow_style, border_style) = match theme {
        UiStyle::AquaGlass => (
            "linear-gradient(135deg, #667eea 0%, #764ba2 100%)".to_string(),
            "0 8px 32px rgba(0,0,0,0.3)".to_string(),
            format!("border-radius: {};", style_sheet.radius_large),
        ),
        UiStyle::SonicFlux => (
            "linear-gradient(135deg, #0a0a0f 0%, #1a1a2f 100%)".to_string(),
            "0 0 15px var(--primary)".to_string(),
            "border: 2px solid var(--primary); border-radius: 0; transform: rotate(45deg);".to_string(),
        ),
        UiStyle::VintagePro => (
            "linear-gradient(180deg, #2d1f15 0%, #1a0f0a 100%)".to_string(),
            "inset 0 0 8px rgba(0,0,0,0.5), 0 2px 4px rgba(0,0,0,0.4)".to_string(),
            "border: 3px solid var(--secondary); border-radius: 2px;".to_string(),
        ),
    };

    // Animation
    let animation = match theme {
        UiStyle::AquaGlass if cx.props.is_playing() => "spin 8s linear infinite",
        _ => "none",
    };

    let inner_style = match theme {
        UiStyle::SonicFlux => {
            // Inner content rotated back for Sonic Flux
            "transform: rotate(-45deg); width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;"
        }
        _ => "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;",
    };

    rsx! {
        div {
            style: "
                width: {size}px;
                height: {size}px;
                background: {bg_gradient};
                box-shadow: {shadow_style};
                {border_style}
                overflow: hidden;
                position: relative;
                animation: {animation};
                {if let UiStyle::AquaGlass = theme {format!("backdrop-filter: {};", style_sheet.glass_blur)} else {String::new()}
            ",
            // Cover image or placeholder
            if let Some(url) = cx.props.cover_url.as_ref() {
                img {
                    src: "{url}",
                    style: "
                        width: 100%;
                        height: 100%;
                        object-fit: cover;
                        {inner_style}
                    ",
                    alt: "Album Cover"
                }
            } else {
                div {
                    style: "
                        width: 100%;
                        height: 100%;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: {size / 3.0}px;
                        color: rgba(255,255,255,0.4);
                        {inner_style}
                    ",
                    "🎵"
                }
            }

            // Sonic Flux: add glow border effect
            if let UiStyle::SonicFlux = theme {
                div {
                    style: "
                        position: absolute;
                        top: -2px; left: -2px; right: -2px; bottom: -2px;
                        border: 2px solid var(--primary);
                        border-radius: 0;
                        pointer-events: none;
                        box-shadow: 0 0 10px var(--primary), inset 0 0 10px var(--primary);
                        animation: flicker 2s infinite;
                    "
                }
            }
        }
    }
}