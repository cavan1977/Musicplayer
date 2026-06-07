 // Theme system - manages UI style switching
 use serde::{Deserialize, Serialize};
 #[allow(unused_imports)]
 use dioxus::prelude::*;
 #[allow(dead_code)]

// ============================================
// StyleSheet - CSS Variables Container
// ============================================

#[derive(Clone, PartialEq)]
pub struct StyleSheet {
    // Colors
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub bg_start: String,
    pub bg_end: String,
    pub glass_bg: String,
    pub text_primary: String,
    pub text_secondary: String,
    // Border radius
    pub radius_large: String,
    pub radius_medium: String,
    pub radius_small: String,
    // Effects
    pub glass_blur: String,
    pub neon_glow: String,
    pub shadow: String,
    // Inline CSS snippets
    pub title_bar_style: String,
    pub button_style: String,
    pub slider_style: String,
    pub vu_style: String,
    // Typography
    pub font_family: String,
    // Animations (raw CSS)
    pub animations: String,
}

impl StyleSheet {
    pub fn to_css_vars(&self) -> String {
        format!(
            r#"
:root {{
    --primary: {};
    --secondary: {};
    --accent: {};
    --color-primary: {};
    --color-secondary: {};
    --color-accent: {};
    --bg-start: {};
    --bg-end: {};
    --glass-bg: {};
    --bg-glass: {};
    --text-primary: {};
    --text-secondary: {};
    --radius-lg: {};
    --radius-md: {};
    --radius-sm: {};
    --glass-blur: {};
    --neon-glow: {};
    --shadow: {};
    --font-family: {};
}}
"#,
            self.primary,
            self.secondary,
            self.accent,
            self.primary,
            self.secondary,
            self.accent,
            self.bg_start,
            self.bg_end,
            self.glass_bg,
            self.glass_bg,
            self.text_primary,
            self.text_secondary,
            self.radius_large,
            self.radius_medium,
            self.radius_small,
            self.glass_blur,
            self.neon_glow,
            self.shadow,
            self.font_family
        )
    }
}

// ============================================
// UiStyle Enum
// ============================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStyle {
    AquaGlass,   // Style A: Organic Fluid
    SonicFlux,  // Style B: Cyberpunk Tech
    VintagePro, // Style C: Retro HiFi
}

 #[allow(dead_code)]
 impl UiStyle {
    pub fn all() -> Vec<UiStyle> {
        vec![
            UiStyle::AquaGlass,
            UiStyle::SonicFlux,
            UiStyle::VintagePro,
        ]
    }
    
    pub fn from_index(idx: u8) -> Option<UiStyle> {
        match idx {
            1 => Some(UiStyle::AquaGlass),
            2 => Some(UiStyle::SonicFlux),
            3 => Some(UiStyle::VintagePro),
            _ => None,
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            UiStyle::AquaGlass => "AQUA GLASS",
            UiStyle::SonicFlux => "SONIC FLUX",
            UiStyle::VintagePro => "VINTAGE PRO",
        }
    }
    
    pub fn shortcut(&self) -> u8 {
        match self {
            UiStyle::AquaGlass => 1,
            UiStyle::SonicFlux => 2,
            UiStyle::VintagePro => 3,
        }
    }
}

// ============================================
// ThemeManager - Plain data, owned inside a Signal
// ============================================

#[derive(Clone)]
pub struct ThemeManager {
    pub current: UiStyle,
    pub style_sheet: StyleSheet,
}

impl ThemeManager {
    pub fn new(initial_style: UiStyle) -> Self {
        Self {
            current: initial_style,
            style_sheet: get_stylesheet(initial_style),
        }
    }
    
    pub fn switch(&mut self, new_style: UiStyle) {
        if self.current != new_style {
            self.current = new_style;
            self.style_sheet = get_stylesheet(new_style);
            save_style_preference(new_style);
        }
    }
}

// ============================================
// Persistence
// ============================================
 
 #[allow(dead_code)]
 const STYLE_PREF_KEY: &str = "ui_style";
 
 #[allow(dead_code)]
 fn save_style_preference(style: UiStyle) {
     let _ = style;
 }

 #[allow(dead_code)]
 fn load_saved_style() -> UiStyle {
     UiStyle::AquaGlass
}

// ============================================
// Style Definitions (inline)
// ============================================

fn create_aqua_glass() -> StyleSheet {
    StyleSheet {
        primary: "#667eea".to_string(),
        secondary: "#764ba2".to_string(),
        accent: "#64ffda".to_string(),
        bg_start: "linear-gradient(135deg, #0f0c29 0%, #302b63 50%, #24243e 100%)".to_string(),
        bg_end: "linear-gradient(135deg, #0f0c29 0%, #302b63 50%, #24243e 100%)".to_string(),
        glass_bg: "rgba(255,255,255,0.06)".to_string(),
        text_primary: "#ffffff".to_string(),
        text_secondary: "rgba(255,255,255,0.6)".to_string(),
        radius_large: "24px".to_string(),
        radius_medium: "14px".to_string(),
        radius_small: "8px".to_string(),
        glass_blur: "blur(28px) saturate(1.2)".to_string(),
        neon_glow: "0 0 20px rgba(100,255,218,0.3)".to_string(),
        shadow: "0 8px 32px rgba(0,0,0,0.3), 0 0 12px rgba(100,255,218,0.1)".to_string(),
        title_bar_style: "height: 48px; background: rgba(255,255,255,0.06); backdrop-filter: blur(24px); border-bottom: 1px solid rgba(255,255,255,0.08); display: flex; justify-content: space-between; align-items: center; padding: 0 16px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);".to_string(),
        button_style: "padding: 10px 20px; background: rgba(255,255,255,0.12); border: 1px solid rgba(255,255,255,0.2); border-radius: var(--radius-md); color: var(--text-primary); cursor: pointer; backdrop-filter: blur(12px); transition: all 0.2s ease; box-shadow: 0 4px 12px rgba(0,0,0,0.1);".to_string(),
        slider_style: "-webkit-appearance: none; width: 100%; height: 8px; border-radius: 4px; background: rgba(255,255,255,0.15); outline: none; box-shadow: inset 0 2px 4px rgba(0,0,0,0.1);".to_string(),
        vu_style: "display: flex; gap: 2px; align-items: flex-end; filter: drop-shadow(0 0 4px var(--accent));".to_string(),
        font_family: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif".to_string(),
        animations: "@keyframes rotate { from { transform: rotate(0deg); } to { transform: rotate(360deg); } } @keyframes vinylSpin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } } @keyframes shimmer { 0% { background-position: -200% 0; } 100% { background-position: 200% 0; } } @keyframes glowPulse { 0%, 100% { box-shadow: 0 0 20px rgba(100,255,218,0.2); } 50% { box-shadow: 0 0 30px rgba(100,255,218,0.4); } } @keyframes floatUp { 0%, 100% { transform: translateY(0px); } 50% { transform: translateY(-6px); } } @keyframes breathe { 0%, 100% { opacity: 0.3; } 50% { opacity: 0.6; } }".to_string(),
    }
}

fn create_sonic_flux() -> StyleSheet {
    StyleSheet {
        primary: "#00f0ff".to_string(),
        secondary: "#ff00ff".to_string(),
        accent: "#00ff9d".to_string(),
        bg_start: "linear-gradient(135deg, #050510 0%, #0a0a1a 40%, #0d1117 100%)".to_string(),
        bg_end: "linear-gradient(135deg, #050510 0%, #0a0a1a 40%, #0d1117 100%)".to_string(),
        glass_bg: "rgba(0,240,255,0.04)".to_string(),
        text_primary: "#e6e6e6".to_string(),
        text_secondary: "rgba(255,255,255,0.5)".to_string(),
        radius_large: "4px".to_string(),
        radius_medium: "2px".to_string(),
        radius_small: "0px".to_string(),
        glass_blur: "blur(12px)".to_string(),
        neon_glow: "0 0 10px var(--primary), 0 0 20px var(--accent), 0 0 30px var(--primary)".to_string(),
        shadow: "0 0 25px rgba(0,240,255,0.4), 0 0 50px rgba(0,255,157,0.2)".to_string(),
        title_bar_style: "height: 60px; background: rgba(10,10,10,0.95); border-bottom: 1px solid rgba(0,240,255,0.4); display: flex; justify-content: space-between; align-items: center; padding: 0 16px; box-shadow: 0 2px 12px rgba(0,240,255,0.2);".to_string(),
        button_style: "padding: 8px 16px; background: rgba(0,240,255,0.1); border: 1px solid var(--primary); color: var(--primary); border-radius: 0; cursor: pointer; transition: all 0.15s; box-shadow: 0 0 8px rgba(0,240,255,0.3); text-shadow: 0 0 4px var(--primary);".to_string(),
        slider_style: "-webkit-appearance: none; width: 100%; height: 4px; border-radius: 0; background: rgba(0,240,255,0.3); outline: none; box-shadow: 0 0 6px rgba(0,240,255,0.5);".to_string(),
        vu_style: "display: flex; gap: 1px; align-items: flex-end; filter: drop-shadow(0 0 6px var(--accent));".to_string(),
        font_family: "'JetBrains Mono', 'Consolas', 'Courier New', monospace".to_string(),
        animations: "@keyframes scan { 0% { transform: translateY(-100%); opacity: 0; } 50% { opacity: 1; } 100% { transform: translateY(100%); opacity: 0; } } @keyframes glitch { 0% { transform: translate(0); } 20% { transform: translate(-2px, 2px); } 40% { transform: translate(-2px, -2px); } 60% { transform: translate(2px, 2px); } 80% { transform: translate(2px, -2px); } 100% { transform: translate(0); } } @keyframes neonPulse { 0%, 100% { box-shadow: 0 0 8px var(--primary), 0 0 16px var(--accent); } 50% { box-shadow: 0 0 16px var(--primary), 0 0 32px var(--accent); } } @keyframes scanline { 0% { top: -10%; } 100% { top: 110%; } } @keyframes dataStream { 0% { background-position: 0 0; } 100% { background-position: 0 100%; } }".to_string(),
    }
}

fn create_vintage_pro() -> StyleSheet {
    StyleSheet {
        primary: "#d4a574".to_string(),
        secondary: "#8b5a2b".to_string(),
        accent: "#ffcc7a".to_string(),
        bg_start: "linear-gradient(135deg, #1a1510 0%, #2c2416 40%, #1e1a12 100%)".to_string(),
        bg_end: "linear-gradient(135deg, #1a1510 0%, #2c2416 40%, #1e1a12 100%)".to_string(),
        glass_bg: "rgba(212,165,116,0.08)".to_string(),
        text_primary: "#f5f1e6".to_string(),
        text_secondary: "rgba(245,241,230,0.6)".to_string(),
        radius_large: "12px".to_string(),
        radius_medium: "6px".to_string(),
        radius_small: "3px".to_string(),
        glass_blur: "blur(12px)".to_string(),
        neon_glow: "0 0 6px rgba(255,204,122,0.3)".to_string(),
        shadow: "0 4px 16px rgba(0,0,0,0.5), 0 0 10px rgba(212,165,116,0.15)".to_string(),
        title_bar_style: "height: 52px; background: rgba(44,36,22,0.95); border-bottom: 1px solid rgba(212,165,116,0.3); display: flex; justify-content: space-between; align-items: center; padding: 0 16px; box-shadow: 0 2px 10px rgba(0,0,0,0.3);".to_string(),
        button_style: "padding: 8px 16px; background: rgba(212,165,116,0.25); border: 1px solid var(--primary); color: var(--text-primary); border-radius: var(--radius-sm); cursor: pointer; transition: all 0.2s; box-shadow: 0 0 6px rgba(212,165,116,0.2); text-shadow: 0 0 3px var(--accent);".to_string(),
        slider_style: "-webkit-appearance: none; width: 100%; height: 6px; border-radius: 3px; background: rgba(212,165,116,0.25); outline: none; box-shadow: inset 0 1px 3px rgba(0,0,0,0.3);".to_string(),
        vu_style: "display: flex; gap: 2px; align-items: flex-end; filter: drop-shadow(0 0 3px var(--accent));".to_string(),
        font_family: "'Georgia', 'Playfair Display', 'Times New Roman', serif".to_string(),
        animations: "@keyframes warmPulse { 0%, 100% { box-shadow: 0 0 4px var(--accent); } 50% { box-shadow: 0 0 10px var(--accent); } } @keyframes mechanicalTick { 0%, 100% { transform: rotate(0deg); } 25% { transform: rotate(-1deg); } 75% { transform: rotate(1deg); } } @keyframes vinylSpin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } } @keyframes warmGlow { 0%, 100% { opacity: 0.6; } 50% { opacity: 1; } }".to_string(),
    }
}

fn get_stylesheet(style: UiStyle) -> StyleSheet {
    match style {
        UiStyle::AquaGlass => create_aqua_glass(),
        UiStyle::SonicFlux => create_sonic_flux(),
        UiStyle::VintagePro => create_vintage_pro(),
    }
}