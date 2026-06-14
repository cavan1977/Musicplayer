// ui/src/components/app.rs
use dioxus::prelude::*;
use crate::Theme;

#[derive(Props, Clone, PartialEq)]
pub struct AppProps {
    pub theme: Theme,
}

pub fn App(cx: Scope<AppProps>) -> Element {
    let theme = cx.props.theme;

    cx.render(rsx! {
        style { r#"
            :root {
                --bg-primary: #1a1a1a;
                --text-primary: #ffffff;
                --accent-color: #ff6b6b;
            }
            .theme-light {
                --bg-primary: #f5f5f5;
                --text-primary: #1a1a1a;
            }
            .app-container {
                background-color: var(--bg-primary);
                color: var(--text-primary);
                transition: background-color 0.5s ease;
            }
        "# }
        div {
            class: "app-container {theme.to_css()}",
            PlayerLayout {}
        }
    })
}

#[derive(Props, Clone, PartialEq)]
pub struct PlayerLayoutProps;

pub fn PlayerLayout(cx: Scope<PlayerLayoutProps>) -> Element {
    cx.render(rsx! {
        div {
            class: "player-layout",
            // TODO: Add components for song list, controls, etc.
        }
    })
}