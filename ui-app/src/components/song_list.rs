// Song List Component - Playlist Display (3 Styles)
use dioxus::prelude::*;
use crate::theme::{use_theme, UiStyle};
use db::Song;

#[derive(Props, Clone)]
pub struct SongListProps {
    pub songs: Signal<Vec<Song>>,
    pub on_select: Callback<Song>,
    pub current_song: Signal<Option<Song>>,
}

pub fn SongList(cx: Scope<SongListProps>) -> Element {
    let theme_manager = use_theme(&cx);
    let theme = theme_manager.current();
    let style_sheet = theme_manager.style_sheet.read();

    let songs = cx.props.songs.read();
    let current = cx.props.current_song(); // Signal read

    let item_bg = match theme {
        UiStyle::AquaGlass => "rgba(255,255,255,0.05)",
        UiStyle::SonicFlux => "rgba(0,0,0,0.4)",
        UiStyle::VintagePro => "rgba(255,255,255,0.02)",
    };

    let item_bg = match theme {
        UiStyle::AquaGlass => "rgba(255,255,255,0.05)",
        UiStyle::SonicFlux => "rgba(0,0,0,0.4)",
        UiStyle::VintagePro => "rgba(255,255,255,0.02)",
    };

    let item_hover_bg = match theme {
        UiStyle::AquaGlass => "rgba(255,255,255,0.1)",
        UiStyle::SonicFlux => "rgba(0,255,255,0.05)",
        UiStyle::VintagePro => "rgba(184,134,11,0.05)",
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 6px;",
            for song in songs.iter() {
                let is_current = current.as_ref().map(|c| c.id == song.id).unwrap_or(false);
                let mut style = String::from("
                    padding: 12px 16px;
                    display: flex;
                    align-items: center;
                    gap: 12px;
                    cursor: pointer;
                    transition: background 0.2s;
                ");
                style.push_str(&format!(" background: {};", item_bg));
                if is_current {
                    style.push_str(&format!(" border-left: 3px solid var(--accent);",));
                }
                match theme {
                    UiStyle::AquaGlass => style.push_str(" border-radius: var(--radius-lg);"),
                    UiStyle::VintagePro => style.push_str(" border: 1px solid var(--secondary); border-radius: 2px;"),
                    _ => style.push_str(" border-radius: 0;"),
                }

                div {
                    key: "{song.id}",
                    style: style,
                    onmouseover: move |e| {
                        let mut new_style = item_bg.to_string();
                        if is_current { new_style.push_str(" border-left: 3px solid var(--accent);"); }
                        new_style.push_str(&format!(" {}", item_hover_bg));
                        match theme {
                            UiStyle::AquaGlass => new_style.push_str(" border-radius: var(--radius-lg);"),
                            UiStyle::VintagePro => new_style.push_str(" border: 1px solid var(--secondary); border-radius: 2px;"),
                            _ => new_style.push_str(" border-radius: 0;"),
                        }
                        e.target.set_attribute("style", &new_style);
                    },
                    onmouseout: move |e| e.target.set_attribute("style", &style),
                    onclick: move |_| cx.props.on_select.call(song.clone()),
                    
                    // Cover art (small)
                    div {
                        style: "
                            width: 40px;
                            height: 40px;
                            background: linear-gradient(135deg, var(--primary), var(--secondary));
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            font-size: 16px;
                            color: white;
                            flex-shrink: 0;
                            {if let UiStyle::AquaGlass = theme {format!("border-radius: {};", style_sheet.radius_small)} else if let UiStyle::VintagePro = theme {"border-radius: 2px;"} else {"border-radius: 0;"}}
                        ",
                        "🎵"
                    }
                    
                    // Title & Artist
                    div {
                        style: "flex: 1; min-width: 0;",
                        div {
                            style: "
                                font-weight: {if is_current {"600"} else {"400"}};
                                margin-bottom: 2px;
                                color: var(--text-primary);
                                white-space: nowrap;
                                overflow: hidden;
                                text-overflow: ellipsis;
                            ",
                            "{song.title}"
                        }
                        div {
                            style: "font-size: 12px; color: var(--text-secondary);",
                            "{song.artist}"
                        }
                    }
                    
                    // Duration
                    div {
                        style: "font-size: 12px; color: var(--text-secondary);",
                        "{format_time(song.duration_secs)}"
                    }
                }
            }
        }
    }
}