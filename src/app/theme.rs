//! Theme management for dark/light mode

use eframe::egui;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub mode: ThemeMode,
    pub accent_color: [u8; 3],
    pub font_size: f32,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Dark,
            accent_color: [0, 200, 100], // Green
            font_size: 14.0,
        }
    }
}

impl ThemeSettings {
    pub fn apply(&self, ctx: &egui::Context) {
        let visuals = match self.mode {
            ThemeMode::Dark => dark_visuals(&self.accent_color),
            ThemeMode::Light => light_visuals(&self.accent_color),
            ThemeMode::System => {
                // Default to dark for now
                dark_visuals(&self.accent_color)
            }
        };
        
        ctx.set_visuals(visuals);
        
        // Apply font size
        let mut style = (*ctx.style()).clone();
        style.text_styles.iter_mut().for_each(|(_, font_id)| {
            font_id.size = self.font_size;
        });
        ctx.set_style(style);
    }

    pub fn accent_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(
            self.accent_color[0],
            self.accent_color[1],
            self.accent_color[2],
        )
    }

    pub fn set_accent(&mut self, color: egui::Color32) {
        self.accent_color = [color.r(), color.g(), color.b()];
    }
}

fn dark_visuals(accent: &[u8; 3]) -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    
    let accent_color = egui::Color32::from_rgb(accent[0], accent[1], accent[2]);
    
    visuals.selection.bg_fill = accent_color;
    visuals.hyperlink_color = accent_color;
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 70);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(70, 70, 80);
    
    // Softer background
    visuals.panel_fill = egui::Color32::from_rgb(30, 30, 35);
    visuals.window_fill = egui::Color32::from_rgb(35, 35, 40);
    visuals.extreme_bg_color = egui::Color32::from_rgb(20, 20, 25);
    
    visuals
}

fn light_visuals(accent: &[u8; 3]) -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    
    let accent_color = egui::Color32::from_rgb(accent[0], accent[1], accent[2]);
    
    visuals.selection.bg_fill = accent_color;
    visuals.hyperlink_color = accent_color;
    
    // Softer background
    visuals.panel_fill = egui::Color32::from_rgb(245, 245, 250);
    visuals.window_fill = egui::Color32::from_rgb(250, 250, 255);
    
    visuals
}

/// Theme picker UI
pub fn theme_picker(ui: &mut egui::Ui, settings: &mut ThemeSettings) -> bool {
    let mut changed = false;
    
    ui.horizontal(|ui| {
        ui.label("Theme:");
        
        if ui.selectable_label(settings.mode == ThemeMode::Dark, "🌙 Dark").clicked() {
            settings.mode = ThemeMode::Dark;
            changed = true;
        }
        
        if ui.selectable_label(settings.mode == ThemeMode::Light, "☀️ Light").clicked() {
            settings.mode = ThemeMode::Light;
            changed = true;
        }
    });
    
    ui.horizontal(|ui| {
        ui.label("Accent:");
        
        let colors = [
            ([0, 200, 100], "Green"),
            ([100, 150, 255], "Blue"),
            ([255, 150, 100], "Orange"),
            ([200, 100, 200], "Purple"),
            ([255, 200, 100], "Gold"),
        ];
        
        for (color, name) in &colors {
            let is_selected = settings.accent_color == *color;
            let btn_color = egui::Color32::from_rgb(color[0], color[1], color[2]);
            
            if ui.add(
                egui::Button::new("")
                    .fill(btn_color)
                    .min_size(egui::vec2(20.0, 20.0))
                    .frame(is_selected)
            ).on_hover_text(*name).clicked() {
                settings.accent_color = *color;
                changed = true;
            }
        }
    });
    
    ui.horizontal(|ui| {
        ui.label("Font size:");
        if ui.add(egui::Slider::new(&mut settings.font_size, 10.0..=20.0)).changed() {
            changed = true;
        }
    });
    
    changed
}

/// Preset themes
pub fn preset_themes() -> Vec<(&'static str, ThemeSettings)> {
    vec![
        ("Default Dark", ThemeSettings {
            mode: ThemeMode::Dark,
            accent_color: [0, 200, 100],
            font_size: 14.0,
        }),
        ("Ocean", ThemeSettings {
            mode: ThemeMode::Dark,
            accent_color: [100, 150, 255],
            font_size: 14.0,
        }),
        ("Sunset", ThemeSettings {
            mode: ThemeMode::Dark,
            accent_color: [255, 150, 100],
            font_size: 14.0,
        }),
        ("Clean Light", ThemeSettings {
            mode: ThemeMode::Light,
            accent_color: [0, 150, 200],
            font_size: 14.0,
        }),
    ]
}
