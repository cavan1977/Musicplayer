//! Music Player UI Components

pub mod components;
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::db::Database;
use crate::core::playback::PlaybackController;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn to_css(&self) -> &'static str {
        match self {
            Theme::Light => "theme-light",
            Theme::Dark => "theme-dark",
        }
    }
}

/// 全局应用状态上下文
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub playback: Arc<Mutex<dyn PlaybackController + Send>>,
    pub theme: Theme,
}

impl AppState {
    pub fn new(db: Database, playback: Arc<Mutex<dyn PlaybackController + Send>>) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            playback,
            theme: Theme::Dark, // 默认深色主题
        }
    }
}

/// 提供 AppState 给 Dioxus 组件
pub fn use_app_state(cx: &Scope) -> &AppState {
    cx.use_hook(|| {
        // TODO：从持久化存储加载主题
        AppState::new(
            Database::open("musicplayer.db").unwrap(),
            // Placeholder：实际初始化播放器
            Arc::new(Mutex::new(crate::core::playback::FFmpegDecoder::new())),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_css() {
        assert_eq!(Theme::Dark.to_css(), "theme-dark");
    }
}
