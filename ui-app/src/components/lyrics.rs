// Lyrics Component - Scrolling Lyrics Synchronization
use dioxus::prelude::*;
use crate::theme::{use_theme, UiStyle};

#[derive(Props, Clone)]
pub struct LyricsProps {
    pub lines: Vec<LyricLine>, // (timestamp, text)
    pub current_time: Signal<f64>, // current playback time in seconds
}

#[derive(Clone)]
pub struct LyricLine {
    pub time: f64, // seconds
    pub text: String,
}

pub fn Lyrics(cx: Scope<LyricsProps>) -> Element {
    let theme_manager = use_theme(&cx);
    let theme = theme_manager.current();
    let style_sheet = theme_manager.style_sheet.read();

    let current_time = cx.props.current_time();
    let lines = cx.props.lines.read(); // Read signal to get reference to Vec

    // Find current lyric line index
    let current_idx = lines.iter()
        .rposition(|line| line.time <= current_time)
        .unwrap_or(0);

    // Container styles
    let container_style = match theme {
        UiStyle::AquaGlass => format!(
            "height: 300px; overflow: hidden; position: relative; padding: 20px 0; background: {}; backdrop-filter: {}; border-radius: {};",
            style_sheet.glass_bg, style_sheet.glass_blur, style_sheet.radius_large
        ),
        UiStyle::SonicFlux => format!(
            "height: 300px; overflow: hidden; position: relative; padding: 20px 0; background: rgba(0,0,0,0.8); border: 1px solid var(--primary); border-radius: 0; box-shadow: inset 0 0 20px rgba(0,255,255,0.2);"
        ),
        UiStyle::VintagePro => format!(
            "height: 300px; overflow: hidden; position: relative; padding: 20px 0; background: rgba(0,0,0,0.3); border: 1px solid var(--secondary); border-radius: 2px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.5);"
        ),
    };

    // Line height
    let line_height = 36.0;

    // Translation based on current line
    let offset_y = -(current_idx as f64 * line_height);

    // Active line style
    let (active_color, active_size, active_weight) = match theme {
        UiStyle::AquaGlass => (style_sheet.accent.as_str(), "18px", "600"),
        UiStyle::SonicFlux => (style_sheet.primary.as_str(), "16px", "700"),
        UiStyle::VintagePro => (style_sheet.accent.as_str(), "16px", "600"),
    };

    rsx! {
        div {
            style: container_style,
            // Lyrics container with smooth scroll
            div {
                style: format!("
                    transform: translateY({}px);
                    transition: transform 0.3s ease-out;
                    display: flex;
                    flex-direction: column;
                    gap: 0px;
                ", offset_y),
                // All lines
                for (idx, line) in lines.iter().enumerate() {
                    let is_active = idx == current_idx;
                    div {
                        key: idx,
                        style: format!(
                            "height: {}px; line-height: {}px; padding: 0 16px; color: {}; font-size: {}; font-weight: {}; opacity: {}; transition: all 0.3s;",
                            line_height, line_height,
                            if is_active { active_color } else { &style_sheet.text_secondary },
                            if is_active { active_size } else { "14px" },
                            if is_active { active_weight } else { "400" },
                            if is_active { "1.0" } else { "0.4" }
                        ),
                        "{line.text}"
                    }
                }
            }
        }
    }
}