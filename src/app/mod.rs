pub mod state;
pub mod ui;
pub mod history;
pub mod theme;
pub mod io;

use eframe::egui;
use state::AppState;
use theme::ThemeSettings;

pub struct App {
    state: AppState,
    theme: ThemeSettings,
    theme_applied: bool,
}

impl Default for App {
    fn default() -> Self {
        Self { 
            state: AppState::default(),
            theme: ThemeSettings::default(),
            theme_applied: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme on first frame or when changed
        if !self.theme_applied {
            self.theme.apply(ctx);
            self.theme_applied = true;
        }

        // Process worker messages first
        self.state.process_worker_messages();

        // Request repaint if generating (to update progress bar)
        if self.state.is_generating {
            ctx.request_repaint();
        }

        // Render UI
        ui::top_bar::render(ctx, &mut self.state, &mut self.theme);
        ui::side_panel::render(ctx, &mut self.state);
        ui::candidates::render(ctx, &mut self.state);

        // Apply theme if changed
        if self.state.theme_changed {
            self.theme.apply(ctx);
            self.state.theme_changed = false;
        }
    }
}
