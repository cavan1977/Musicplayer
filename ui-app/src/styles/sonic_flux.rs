// Style B: SONIC FLUX - Cyberpunk Tech Design
use crate::theme::StyleSheet;

pub fn create() -> StyleSheet {
    StyleSheet {
        // Colors from design doc
        primary: "#00ffff".to_string(),
        secondary: "#ff00ff".to_string(),
        accent: "#ffff00".to_string(),
        bg_start: "#0a0a0f".to_string(),
        bg_end: "#0a0a0f".to_string(),
        glass_bg: "rgba(0,0,0,0.9)".to_string(),
        text_primary: "#ffffff".to_string(),
        text_secondary: "#888888".to_string(),
        
        radius_large: "0px".to_string(),
        radius_medium: "2px".to_string(),
        radius_small: "2px".to_string(),
        
        glass_blur: "none".to_string(),
        neon_glow: "0 0 10px var(--primary), inset 0 0 5px var(--primary)".to_string(),
        shadow: "none".to_string(),
        
        title_bar_style: "
            height: 40px;
            background: var(--bg-start);
            border-bottom: 1px solid var(--primary);
            box-shadow: 0 0 10px var(--primary);
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0 16px;
        ".to_string(),
        
        button_style: "
            padding: 10px 16px;
            background: transparent;
            border: 1px solid var(--primary);
            border-radius: 0;
            color: var(--primary);
            cursor: pointer;
            font-family: var(--font-family);
            text-transform: uppercase;
            letter-spacing: 1px;
            box-shadow: 0 0 5px var(--primary);
        ".to_string(),
        
        slider_style: "
            -webkit-appearance: none;
            width: 100%;
            height: 8px;
            border-radius: 0;
            background: rgba(0,255,255,0.2);
            outline: none;
        ".to_string(),
        
        vu_style: "
            display: flex;
            gap: 2px;
            align-items: flex-end;
            height: 40px;
        ".to_string(),
        
        font_family: "'Orbitron', 'Rajdhani', monospace".to_string(),
        
         animations: "
             @keyframes scanline {{
                 0% {{ transform: translateY(-100%); }}
                 100% {{ transform: translateY(100%); }}
             }}
             @keyframes flicker {{
                 0%, 100% {{ opacity: 1; }}
                 50% {{ opacity: 0.8; }}
             }}
         ".to_string(),
     }
}