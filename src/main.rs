mod app;
mod chemistry;
mod generation;
mod optimization;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Drug Candidate Studio"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Drug Candidate Studio",
        options,
        Box::new(|_cc| Box::new(app::App::default())),
    )
}
