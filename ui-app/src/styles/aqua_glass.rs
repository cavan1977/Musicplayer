// Style A: AQUA GLASS - Organic Fluid Design
use crate::theme::StyleSheet;

pub fn create() -> StyleSheet {
    StyleSheet {
        // Colors from design doc
        primary: "#667eea".to_string(),
        secondary: "#764ba2".to_string(),
        accent: "#ff6b6b".to_string(),
        bg_start: "linear-gradient(135deg, #667eea 0%, #764ba2 100%)".to_string(),
        bg_end: "linear-gradient(135deg, #667eea 0%, #764ba2 100%)".to_string(),
        glass_bg: "rgba(255,255,255,0.1)".to_string(),
        text_primary: "#ffffff".to_string(),
        text_secondary: "rgba(255,255,255,0.6)".to_string(),
        
        radius_large: "24px".to_string(),
        radius_medium: "12px".to_string(),
        radius_small: "8px".to_string(),
        
        glass_blur: "blur(20px)".to_string(),
        neon_glow: "none".to_string(),
        shadow: "0 8px 32px rgba(0,0,0,0.3)".to_string(),
        
        title_bar_style: "
            height: 40px;
            background: rgba(255,255,255,0.05);
            backdrop-filter: blur(20px);
            border-bottom: 1px solid rgba(255,255,255,0.1);
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0 16px;
        ".to_string(),
        
        button_style: "
            padding: 10px 20px;
            background: rgba(255,255,255,0.15);
            border: 1px solid rgba(255,255,255,0.2);
            border-radius: var(--radius-md);
            color: var(--text-primary);
            cursor: pointer;
            backdrop-filter: blur(10px);
            transition: all 0.2s ease;
        ".to_string(),
        
        slider_style: "
            -webkit-appearance: none;
            width: 100%;
            height: 6px;
            border-radius: 3px;
            background: rgba(255,255,255,0.2);
            outline: none;
        ".to_string(),
        
        vu_style: "
            display: flex;
            gap: 4px;
            align-items: flex-end;
        ".to_string(),
        
        font_family: "'Segoe UI', -apple-system, BlinkMacSystemFont, sans-serif".to_string(),
        
        animations: "
            @keyframes spin {{
                from {{ transform: rotate(0deg); }}
                to {{ transform: rotate(360deg); }}
            }}
            @keyframes pulse {{
                0%, 100% {{ opacity: 0.6; }}
                50% {{ opacity: 1; }}
            }}
        ".to_string(),
    }
}