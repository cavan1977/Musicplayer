// Style C: VINTAGE PRO - Retro HiFi Design
use crate::theme::StyleSheet;

pub fn create() -> StyleSheet {
    StyleSheet {
        // Colors from design doc
        primary: "#b8860b".to_string(),        // Brushed Gold
        secondary: "#cd7f32".to_string(),      // Bronze
        accent: "#ffbf00".to_string(),         // Warm Amber
        bg_start: "#1a0f0a".to_string(),       // Walnut dark
        bg_end: "#2d1f15".to_string(),         // Oak
        glass_bg: "rgba(0,0,0,0.4)".to_string(),
        text_primary: "#e0d0c0".to_string(),  // Warm white
        text_secondary: "#8b7355".to_string(),
        
        radius_large: "4px".to_string(),
        radius_medium: "4px".to_string(),
        radius_small: "2px".to_string(),
        
        glass_blur: "none".to_string(),
        neon_glow: "none".to_string(),
        shadow: "inset 0 0 8px rgba(0,0,0,0.5)".to_string(),
        
        title_bar_style: "
            height: 40px;
            background: linear-gradient(180deg, #4a3a2a 0%, #2d1f15 100%);
            border-bottom: 2px solid var(--secondary);
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0 16px;
        ".to_string(),
        
        button_style: "
            padding: 10px 16px;
            background: linear-gradient(180deg, #8b6914 0%, #5a4510 100%);
            border: 1px solid var(--secondary);
            border-radius: var(--radius-sm);
            color: var(--text-primary);
            cursor: pointer;
            font-family: var(--font-family);
            box-shadow: inset 0 1px 0 rgba(255,255,255,0.2);
        ".to_string(),
        
        slider_style: "
            -webkit-appearance: none;
            width: 100%;
            height: 12px;
            border-radius: 6px;
            background: #1a0f0a;
            outline: none;
            border: 1px solid var(--secondary);
        ".to_string(),
        
        vu_style: "
            display: flex;
            gap: 2px;
            align-items: flex-end;
            height: 30px;
            padding: 4px;
            background: #0a0705;
            border: 1px solid var(--secondary);
            border-radius: var(--radius-small);
        ".to_string(),
        
        font_family: "'DIN Alternate', 'Franklin Gothic', sans-serif".to_string(),
        
        animations: "
            @keyframes led-glow {{
                0%, 100% {{ box-shadow: 0 0 5px #39ff14; }}
                50% {{ box-shadow: 0 0 10px #39ff14; }}
            }}
        ".to_string(),
    }
}