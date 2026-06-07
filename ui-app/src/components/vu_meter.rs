// VU Meter Component - 3 Distinct Visualizations
use dioxus::prelude::*;
use crate::theme::{use_theme, UiStyle};

#[derive(Props, Clone)]
pub struct VUMeterProps {
    pub peak: Signal<f32>, // 0.0 - 1.0
    pub style: Option<UiStyle>,
}

pub fn VUMeter(cx: Scope<VUMeterProps>) -> Element {
    let theme_manager = use_theme(&cx);
    let theme = cx.props.style.unwrap_or(theme_manager.current());
    let style_sheet = theme_manager.style_sheet.read();
    let peak = cx.props.peak();

    match theme {
        UiStyle::AquaGlass => {
            // 5 circular LEDs
            let bars = (0..5).map(|i| {
                let intensity = (peak * 5.0).clamp(0.0, 1.0);
                let active = i as f32 / 5.0 < intensity;
                let color = if active { "var(--accent)" } else { "rgba(255,255,255,0.2)" };
                rsx! {
                    div {
                        key: i,
                        style: "
                            width: 12px;
                            height: 12px;
                            border-radius: 50%;
                            background: {color};
                            box-shadow: {if active {format!("0 0 10px {}", color)} else {String::new()}};
                            transition: background 0.1s;
                        "
                    }
                }
            }).collect::<Vec<_>>();

            rsx! {
                div {
                    style: "display: flex; gap: 6px; align-items: flex-end; height: 24px;",
                    {bars}
                }
            }
        }
        UiStyle::SonicFlux => {
            // 10-bar spectrum analyzer
            let bars = (0..10).map(|i| {
                let intensity = (peak * 10.0).clamp(0.0, 1.0);
                let active = i as f32 / 10.0 < intensity;
                let color = if active {
                    if i < 3 { "var(--primary)" } else if i < 7 { "var(--accent)" } else { "var(--secondary)" }
                } else {
                    "rgba(255,255,255,0.1)"
                };
                let height = if active { 15 + i * 3 } else { 5 };
                rsx! {
                    div {
                        key: i,
                        style: "
                            flex: 1;
                            height: {height}px;
                            background: {color};
                            border-radius: 1px;
                            box-shadow: 0 0 5px {color};
                            transition: height 0.05s;
                        "
                    }
                }
            }).collect::<Vec<_>>();

            rsx! {
                div {
                    style: "display: flex; gap: 2px; align-items: flex-end; height: 30px; width: 120px;",
                    {bars}
                }
            }
        }
        UiStyle::VintagePro => {
            // Dual-needle analog meter (simplified as two bars)
            let left = (peak * 20.0).clamp(0.0, 10.0);
            let right = (peak * 20.0).clamp(0.0, 10.0);
            let (needle_color, bg_color) = (
                match peak {
                    p if p < 0.6 => "#39ff14", // green
                    p if p < 0.85 => "#ffff00", // yellow
                    _ => "#ff0000", // red
                },
                "#0a0705"
            );

            rsx! {
                div {
                    style: "
                        display: flex;
                        gap: 8px;
                        align-items: center;
                        background: {bg_color};
                        border: 1px solid var(--secondary);
                        border-radius: 2px;
                        padding: 4px;
                        box-shadow: inset 0 1px 2px rgba(0,0,0,0.6);
                    ",
                    // Left channel
                    div {
                        style: "
                            width: 4px;
                            height: 20px;
                            background: #333;
                            border-radius: 2px;
                            position: relative;
                        ",
                        div {
                            style: "
                                position: absolute;
                                bottom: 0;
                                width: 100%;
                                height: {left}px;
                                background: {needle_color};
                                border-radius: 2px;
                                transition: height 0.1s;
                                box-shadow: 0 0 5px {needle_color};
                            "
                        }
                    }
                    // Right channel
                    div {
                        style: "
                            width: 4px;
                            height: 20px;
                            background: #333;
                            border-radius: 2px;
                            position: relative;
                        ",
                        div {
                            style: "
                                position: absolute;
                                bottom: 0;
                                width: 100%;
                                height: {right}px;
                                background: {needle_color};
                                border-radius: 2px;
                                transition: height 0.1s;
                                box-shadow: 0 0 5px {needle_color};
                            "
                        }
                    }
                    // Scale markings (static)
                    div {
                        style: "
                            position: absolute;
                            right: 4px;
                            top: 4px;
                            bottom: 4px;
                            width: 1px;
                            background: var(--secondary);
                            opacity: 0.5;
                        "
                    }
                }
            }
        }
    }
}