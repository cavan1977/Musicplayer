// Custom Title Bar Component
use dioxus::prelude::*;
use dioxus_desktop::use_window;
use crate::theme::{use_theme, UiStyle};

#[derive(Props, Clone)]
pub struct TitleBarProps {
    pub show_settings: Signal<bool>,
}

pub fn TitleBar(cx: Scope<TitleBarProps>) -> Element {
    let window = use_window(&cx);
    let theme_manager = use_theme(&cx);
    let theme = theme_manager.current();
    let style_sheet = theme_manager.style_sheet.read();

    let minimize = move |_| { window.minimize().ok(); };
    let maximize = move |_| {
        if window.is_maximized().unwrap_or(false) { window.restore().ok(); }
        else { window.maximize().ok(); }
    };
    let close = move |_| { window.close().ok(); };
    let open_settings = {
        let show_settings = cx.props.show_settings.clone();
        move |_| { show_settings.set(true); }
    };

    // Title content
    let title_content = match theme {
        UiStyle::AquaGlass => rsx! {
            div { style: "display:flex; align-items:center; gap:8px; font-size:14px; font-weight:500; color:var(--text-secondary);", "🎵", span { style: "font-family: var(--font-family);", "HiFi Music Player" } }
        },
        UiStyle::SonicFlux => rsx! {
            div { style: "display:flex; align-items:center; gap:10px; font-family:var(--font-family); font-size:12px; letter-spacing:2px; color:var(--primary); text-transform:uppercase;", "[ HIFIPLAYER ]" }
        },
        UiStyle::VintagePro => rsx! {
            div { style: "display:flex; align-items:center; gap:10px; font-family:var(--font-family); font-size:14px; font-weight:600; color:var(--text-primary); text-shadow:0 1px 0 rgba(255,255,255,0.2);", "⚡ HI-FI PRO" }
        },
    };

    // Buttons
    let (min_btn, max_btn, close_btn) = match theme {
        UiStyle::AquaGlass => (
            rsx! { button { onclick: minimize, style: "width:28px; height:28px; border-radius:14px; border:none; background:rgba(255,255,255,0.1); color:var(--text-primary); cursor:pointer; display:flex; align-items:center; justify-content:center; font-size:12px;", "−" } },
            rsx! { button { onclick: maximize, style: "width:28px; height:28px; border-radius:14px; border:none; background:rgba(255,255,255,0.1); color:var(--text-primary); cursor:pointer; display:flex; align-items:center; justify-content:center; font-size:12px;", "□" } },
            rsx! { button { onclick: close, style: "width:28px; height:28px; border-radius:14px; border:none; background:rgba(255,0,0,0.3); color:white; cursor:pointer; display:flex; align-items:center; justify-content:center; font-size:14px; font-weight:bold;", "✕" } },
        ),
        UiStyle::SonicFlux => (
            rsx! { button { onclick: minimize, style: "width:24px; height:24px; border:1px solid var(--primary); border-radius:0; background:transparent; color:var(--primary); cursor:pointer; font-family:monospace; font-size:16px; line-height:1; box-shadow:0 0 5px var(--primary);", "_" } },
            rsx! { button { onclick: maximize, style: "width:24px; height:24px; border:1px solid var(--primary); border-radius:0; background:transparent; color:var(--primary); cursor:pointer; font-family:monospace; font-size:14px; box-shadow:0 0 5px var(--primary);", "□" } },
            rsx! { button { onclick: close, style: "width:24px; height:24px; border:1px solid #ff0055; border-radius:0; background:rgba(255,0,85,0.2); color:#ff0055; cursor:pointer; font-size:14px; box-shadow:0 0 8px #ff0055;", "✕" } },
        ),
        UiStyle::VintagePro => (
            rsx! { button { onclick: minimize, style: "width:26px; height:26px; border:1px solid var(--secondary); border-radius:2px; background:linear-gradient(180deg, #8b6914 0%, #5a4510 100%); color:var(--text-primary); cursor:pointer; font-family:var(--font-family); font-size:14px; line-height:1; box-shadow:inset 0 1px 0 rgba(255,255,255,0.2), 0 1px 2px rgba(0,0,0,0.3);", "−" } },
            rsx! { button { onclick: maximize, style: "width:26px; height:26px; border:1px solid var(--secondary); border-radius:2px; background:linear-gradient(180deg, #8b6914 0%, #5a4510 100%); color:var(--text-primary); cursor:pointer; font-family:var(--font-family); font-size:14px; box-shadow:inset 0 1px 0 rgba(255,255,255,0.2), 0 1px 2px rgba(0,0,0,0.3);", "□" } },
            rsx! { button { onclick: close, style: "width:26px; height:26px; border:1px solid #8b0000; border-radius:2px; background:linear-gradient(180deg, #5a0000 0%, #3d0000 100%); color:#ff4444; cursor:pointer; font-family:var(--font-family); font-size:14px; font-weight:bold; box-shadow:inset 0 1px 0 rgba(255,255,255,0.1), 0 1px 2px rgba(0,0,0,0.5);", "✕" } },
        ),
    };

    rsx! {
        div {
            style: format!("height:40px; display:flex; justify-content:space-between; align-items:center; padding:0 16px; user-select:none; {}", style_sheet.title_bar_style),
            title_content,
            div { style: "display:flex; gap:8px; align-items:center;", min_btn, max_btn, close_btn }
        }
    }
}

pub fn TitleBar(cx: Scope<TitleBarProps>) -> Element {
    let window = use_window(&cx);
    let theme_manager = use_theme(&cx);
    let theme = theme_manager.current();
    let style_sheet = theme_manager.style_sheet.read();

    // Window control handlers
    let minimize = move |_| {
        window.minimize().ok();
    };

    let maximize = move |_| {
        if window.is_maximized().unwrap_or(false) {
            window.restore().ok();
        } else {
            window.maximize().ok();
        }
    };

    let close = move |_| {
        window.close().ok();
    };

    let open_settings = {
        let show_settings = cx.props.show_settings.clone();
        move |_| {
            show_settings.set(true);
        }
    };

    // Title bar content based on style
    let title_content = match theme {
        UiStyle::AquaGlass => {
            rsx! {
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        font-size: 14px;
                        font-weight: 500;
                        color: var(--text-secondary);
                    ",
                    "🎵",
                    span {
                        style: "font-family: var(--font-family);",
                        "HiFi Music Player"
                    }
                }
            }
        }
        UiStyle::SonicFlux => {
            rsx! {
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 10px;
                        font-family: var(--font-family);
                        font-size: 12px;
                        letter-spacing: 2px;
                        color: var(--primary);
                        text-transform: uppercase;
                    ",
                    "[ HIFIPLAYER ]"
                }
            }
        }
        UiStyle::VintagePro => {
            rsx! {
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 10px;
                        font-family: var(--font-family);
                        font-size: 14px;
                        font-weight: 600;
                        color: var(--text-primary);
                        text-shadow: 0 1px 0 rgba(255,255,255,0.2);
                    ",
                    "⚡ HI-FI PRO"
                }
            }
        }
    };

    // Window control buttons per style
    let (min_btn, max_btn, close_btn) = match theme {
        UiStyle::AquaGlass => (
            rsx! {
                button {
                    onclick: minimize,
                    style: "
                        width: 28px;
                        height: 28px;
                        border-radius: 14px;
                        border: none;
                        background: rgba(255,255,255,0.1);
                        color: var(--text-primary);
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 12px;
                    ",
                    "−"
                }
            },
            rsx! {
                button {
                    onclick: maximize,
                    style: "
                        width: 28px;
                        height: 28px;
                        border-radius: 14px;
                        border: none;
                        background: rgba(255,255,255,0.1);
                        color: var(--text-primary);
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 12px;
                    ",
                    "□"
                }
            },
            rsx! {
                button {
                    onclick: close,
                    style: "
                        width: 28px;
                        height: 28px;
                        border-radius: 14px;
                        border: none;
                        background: rgba(255,0,0,0.3);
                        color: white;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 14px;
                        font-weight: bold;
                    ",
                    "✕"
                }
            },
        ),
        UiStyle::SonicFlux => (
            rsx! {
                button {
                    onclick: minimize,
                    style: "
                        width: 24px;
                        height: 24px;
                        border: 1px solid var(--primary);
                        border-radius: 0;
                        background: transparent;
                        color: var(--primary);
                        cursor: pointer;
                        font-family: monospace;
                        font-size: 16px;
                        line-height: 1;
                        box-shadow: 0 0 5px var(--primary);
                    ",
                    "_"
                }
            },
            rsx! {
                button {
                    onclick: maximize,
                    style: "
                        width: 24px;
                        height: 24px;
                        border: 1px solid var(--primary);
                        border-radius: 0;
                        background: transparent;
                        color: var(--primary);
                        cursor: pointer;
                        font-family: monospace;
                        font-size: 14px;
                        box-shadow: 0 0 5px var(--primary);
                    ",
                    "□"
                }
            },
            rsx! {
                button {
                    onclick: close,
                    style: "
                        width: 24px;
                        height: 24px;
                        border: 1px solid #ff0055;
                        border-radius: 0;
                        background: rgba(255,0,85,0.2);
                        color: #ff0055;
                        cursor: pointer;
                        font-size: 14px;
                        box-shadow: 0 0 8px #ff0055;
                    ",
                    "✕"
                }
            },
        ),
        UiStyle::VintagePro => (
            rsx! {
                button {
                    onclick: minimize,
                    style: "
                        width: 26px;
                        height: 26px;
                        border: 1px solid var(--secondary);
                        border-radius: 2px;
                        background: linear-gradient(180deg, #8b6914 0%, #5a4510 100%);
                        color: var(--text-primary);
                        cursor: pointer;
                        font-family: var(--font-family);
                        font-size: 14px;
                        line-height: 1;
                        box-shadow: inset 0 1px 0 rgba(255,255,255,0.2), 0 1px 2px rgba(0,0,0,0.3);
                    ",
                    "−"
                }
            },
            rsx! {
                button {
                    onclick: maximize,
                    style: "
                        width: 26px;
                        height: 26px;
                        border: 1px solid var(--secondary);
                        border-radius: 2px;
                        background: linear-gradient(180deg, #8b6914 0%, #5a4510 100%);
                        color: var(--text-primary);
                        cursor: pointer;
                        font-family: var(--font-family);
                        font-size: 14px;
                        box-shadow: inset 0 1px 0 rgba(255,255,255,0.2), 0 1px 2px rgba(0,0,0,0.3);
                    ",
                    "□"
                }
            },
            rsx! {
                button {
                    onclick: close,
                    style: "
                        width: 26px;
                        height: 26px;
                        border: 1px solid #8b0000;
                        border-radius: 2px;
                        background: linear-gradient(180deg, #5a0000 0%, #3d0000 100%);
                        color: #ff4444;
                        cursor: pointer;
                        font-family: var(--font-family);
                        font-size: 14px;
                        font-weight: bold;
                        box-shadow: inset 0 1px 0 rgba(255,255,255,0.1), 0 1px 2px rgba(0,0,0,0.5);
                    ",
                    "✕"
                }
            },
        ),
    };

    rsx! {
        div {
            style: "
                height: 40px;
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 0 16px;
                user-select: none;
                {style_sheet.title_bar_style}
            ",
            // Left: Title
            title_content,
            // Right: Window controls
            div {
                style: "display: flex; gap: 8px; align-items: center;",
                min_btn,
                max_btn,
                close_btn,
            }
        }
    }
}

pub fn TitleBar(cx: Scope<TitleBarProps>) -> Element {
    let window = use_window(&cx);
    let theme_manager = use_theme(&cx);
    let theme = theme_manager.current();
    let style_sheet = theme_manager.style_sheet.read();

    // Window control handlers
    let minimize = move |_| {
        window.minimize().ok();
    };

    let maximize = move |_| {
        if window.is_maximized().unwrap_or(false) {
            window.restore().ok();
        } else {
            window.maximize().ok();
        }
    };

    let close = move |_| {
        window.close().ok();
    };

    let open_settings = move |_| {
        cx.props.show_settings.set(true);
    };

    // Title bar content based on style
    let (title_content, button_style) = match theme {
        UiStyle::AquaGlass => (
            rsx! {
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        font-size: 14px;
                        font-weight: 500;
                        color: var(--text-secondary);
                    ",
                    "🎵"
                    span {
                        style: "font-family: var(--font-family);",
                        "HiFi Music Player"
                    }
                }
            },
            String::new(), // Use inline styles for buttons
        ),
        UiStyle::SonicFlux => (
            rsx! {
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 10px;
                        font-family: var(--font-family);
                        font-size: 12px;
                        letter-spacing: 2px;
                        color: var(--primary);
                        text-transform: uppercase;
                    ",
                    "[ HIFIPLAYER ]"
                }
            },
            String::new(),
        ),
        UiStyle::VintagePro => (
            rsx! {
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 10px;
                        font-family: var(--font-family);
                        font-size: 14px;
                        font-weight: 600;
                        color: var(--text-primary);
                        text-shadow: 0 1px 0 rgba(255,255,255,0.2);
                    ",
                    "⚡ HI-FI PRO"
                }
            },
            String::new(),
        ),
    };

    // Window control buttons per style
    let (min_btn, max_btn, close_btn) = match theme {
        UiStyle::AquaGlass => (
            rsx! {
                button {
                    onclick: minimize,
                    style: "
                        width: 28px;
                        height: 28px;
                        border-radius: 14px;
                        border: none;
                        background: rgba(255,255,255,0.1);
                        color: var(--text-primary);
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 12px;
                        transition: background 0.2s;
                    ",
                    onmouseover: move |_| {
                        // Hover handled via CSS ideally
                    },
                    "−"
                }
            },
            rsx! {
                button {
                    onclick: maximize,
                    style: "
                        width: 28px;
                        height: 28px;
                        border-radius: 14px;
                        border: none;
                        background: rgba(255,255,255,0.1);
                        color: var(--text-primary);
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 12px;
                    ",
                    "□"
                }
            },
            rsx! {
                button {
                    onclick: close,
                    style: "
                        width: 28px;
                        height: 28px;
                        border-radius: 14px;
                        border: none;
                        background: rgba(255,0,0,0.3);
                        color: white;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 14px;
                        font-weight: bold;
                    ",
                    "✕"
                }
            },
        ),
        UiStyle::SonicFlux => (
            rsx! {
                button {
                    onclick: minimize,
                    style: "
                        width: 24px;
                        height: 24px;
                        border: 1px solid var(--primary);
                        border-radius: 0;
                        background: transparent;
                        color: var(--primary);
                        cursor: pointer;
                        font-family: monospace;
                        font-size: 16px;
                        line-height: 1;
                        box-shadow: 0 0 5px var(--primary);
                    ",
                    "_"
                }
            },
            rsx! {
                button {
                    onclick: maximize,
                    style: "
                        width: 24px;
                    height: 24px;
                    border: 1px solid var(--primary);
                    border-radius: 0;
                    background: transparent;
                    color: var(--primary);
                    cursor: pointer;
                    font-family: monospace;
                    font-size: 14px;
                    box-shadow: 0 0 5px var(--primary);
                    ",
                    "□"
                }
            },
            rsx! {
                button {
                    onclick: close,
                    style: "
                        width: 24px;
                        height: 24px;
                        border: 1px solid #ff0055;
                        border-radius: 0;
                        background: rgba(255,0,85,0.2);
                        color: #ff0055;
                        cursor: pointer;
                        font-size: 14px;
                        box-shadow: 0 0 8px #ff0055;
                    ",
                    "✕"
                }
            },
        ),
        UiStyle::VintagePro => (
            rsx! {
                button {
                    onclick: minimize,
                    style: "
                        width: 26px;
                        height: 26px;
                        border: 1px solid var(--secondary);
                        border-radius: 2px;
                        background: linear-gradient(180deg, #8b6914 0%, #5a4510 100%);
                        color: var(--text-primary);
                        cursor: pointer;
                        font-family: var(--font-family);
                        font-size: 14px;
                        line-height: 1;
                        box-shadow: inset 0 1px 0 rgba(255,255,255,0.2), 0 1px 2px rgba(0,0,0,0.3);
                    ",
                    "−"
                }
            },
            rsx! {
                button {
                    onclick: maximize,
                    style: "
                        width: 26px;
                        height: 26px;
                        border: 1px solid var(--secondary);
                        border-radius: 2px;
                        background: linear-gradient(180deg, #8b6914 0%, #5a4510 100%);
                        color: var(--text-primary);
                        cursor: pointer;
                        font-family: var(--font-family);
                        font-size: 14px;
                        box-shadow: inset 0 1px 0 rgba(255,255,255,0.2), 0 1px 2px rgba(0,0,0,0.3);
                    ",
                    "□"
                }
            },
            rsx! {
                button {
                    onclick: close,
                    style: "
                        width: 26px;
                        height: 26px;
                        border: 1px solid #8b0000;
                        border-radius: 2px;
                        background: linear-gradient(180deg, #5a0000 0%, #3d0000 100%);
                        color: #ff4444;
                        cursor: pointer;
                        font-family: var(--font-family);
                        font-size: 14px;
                        font-weight: bold;
                        box-shadow: inset 0 1px 0 rgba(255,255,255,0.1), 0 1px 2px rgba(0,0,0,0.5);
                    ",
                    "✕"
                }
            },
        ),
    };

    rsx! {
        div {
            style: "
                height: 40px;
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 0 16px;
                user-select: none;
                {style_sheet.title_bar_style}
            ",
            // Left: Title
            title_content,
            // Right: Window controls
            div {
                style: "display: flex; gap: 8px; align-items: center;",
                min_btn,
                max_btn,
                close_btn,
            }
        }
    }
}