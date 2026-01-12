use eframe::egui;
use crate::app::state::AppState;
use crate::app::theme::{ThemeSettings, theme_picker};
use crate::app::io;

pub fn render(ctx: &egui::Context, state: &mut AppState, theme: &mut ThemeSettings) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("💊 Drug Candidate Studio");

            ui.separator();
            
            // File menu
            ui.menu_button("📁 File", |ui| {
                if ui.button("💾 Save Session").clicked() {
                    save_session_dialog(state);
                    ui.close_menu();
                }
                if ui.button("📂 Load Session").clicked() {
                    load_session_dialog(state);
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("📥 Import SMILES...").clicked() {
                    state.show_import_dialog = true;
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("📊 Export CSV").clicked() {
                    export_csv(state);
                    ui.close_menu();
                }
                if ui.button("📋 Export JSON").clicked() {
                    export_json(state);
                    ui.close_menu();
                }
                if ui.button("🧬 Export SDF").clicked() {
                    export_sdf(state);
                    ui.close_menu();
                }
                if ui.button("📝 Export SMILES").clicked() {
                    export_smiles(state);
                    ui.close_menu();
                }
            });

            // Edit menu
            ui.menu_button("✏️ Edit", |ui| {
                let undo_text = if let Some(desc) = state.history.last_action_description() {
                    format!("↩️ Undo: {}", desc)
                } else {
                    "↩️ Undo".to_string()
                };
                
                if ui.add_enabled(state.history.can_undo(), egui::Button::new(undo_text)).clicked() {
                    state.undo();
                    ui.close_menu();
                }
                
                if ui.add_enabled(state.history.can_redo(), egui::Button::new("↪️ Redo")).clicked() {
                    state.redo();
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("🗑️ Clear All").clicked() {
                    state.clear();
                    ui.close_menu();
                }
            });

            // View menu
            ui.menu_button("👁 View", |ui| {
                ui.label("📊 Visualizations:");
                ui.checkbox(&mut state.show_histograms, "Histograms");
                ui.checkbox(&mut state.show_parallel_coords, "Parallel Coordinates");
                ui.checkbox(&mut state.show_3d_plot, "3D Plot");
                ui.checkbox(&mut state.show_heatmap, "Correlation Heatmap");
                
                ui.separator();
                
                ui.label("🔬 Analysis:");
                ui.checkbox(&mut state.show_clustering, "Clustering");
                ui.checkbox(&mut state.show_similarity_search, "Similarity Search");
                ui.checkbox(&mut state.show_druglikeness, "Drug-likeness Panel");
            });

            // Settings menu
            ui.menu_button("⚙️ Settings", |ui| {
                ui.label("🎨 Theme:");
                if theme_picker(ui, theme) {
                    state.theme_changed = true;
                }
            });

            ui.separator();

            // Generation controls
            ui.label("Generate:");
            ui.add(egui::DragValue::new(&mut state.n_generate).clamp_range(10..=100_000).speed(10));
            
            ui.label("Seed:");
            ui.add(egui::DragValue::new(&mut state.seed).clamp_range(0..=u64::MAX).speed(1));

            ui.checkbox(&mut state.use_parallel, "⚡").on_hover_text("Parallel generation");
            ui.checkbox(&mut state.use_scaffolds, "💊").on_hover_text("Use drug scaffolds");

            if state.is_generating {
                if ui.button("⏹ Cancel").clicked() {
                    state.cancel_generation();
                }
            } else {
                if ui.button("🧬 Generate").clicked() {
                    state.generate();
                }
            }

            // Undo/Redo buttons
            ui.separator();
            if ui.add_enabled(state.history.can_undo(), egui::Button::new("↩️")).on_hover_text("Undo").clicked() {
                state.undo();
            }
            if ui.add_enabled(state.history.can_redo(), egui::Button::new("↪️")).on_hover_text("Redo").clicked() {
                state.redo();
            }

            ui.separator();
            
            // Status
            let status_color = if state.is_generating {
                egui::Color32::from_rgb(100, 180, 255)
            } else if state.status.contains("Error") || state.status.contains("error") {
                egui::Color32::from_rgb(255, 100, 100)
            } else {
                egui::Color32::from_rgb(100, 255, 100)
            };
            ui.colored_label(status_color, &state.status);

            // Progress bar
            if let Some((current, total)) = state.generation_progress {
                ui.separator();
                let progress = current as f32 / total as f32;
                ui.add(egui::ProgressBar::new(progress).text(format!("{}/{}", current, total)).animate(true));
            }
        });
    });

    // Import dialog window
    render_import_dialog(ctx, state);
}

fn render_import_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_import_dialog {
        return;
    }

    egui::Window::new("📥 Import SMILES")
        .collapsible(false)
        .resizable(true)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.label("Paste SMILES strings (one per line):");
            
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut state.import_text)
                            .desired_width(f32::INFINITY)
                            .desired_rows(10)
                            .font(egui::TextStyle::Monospace)
                    );
                });

            ui.horizontal(|ui| {
                if ui.button("📂 Load from file...").clicked() {
                    // Simple file loading
                    if let Ok(entries) = std::fs::read_dir(".") {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if name.ends_with(".smi") || name.ends_with(".txt") {
                                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                    state.import_text = content;
                                    break;
                                }
                            }
                        }
                    }
                }
                
                ui.label(format!("Lines: {}", state.import_text.lines().count()));
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("✅ Import").clicked() {
                    state.import_from_text(&state.import_text.clone());
                    state.import_text.clear();
                    state.show_import_dialog = false;
                }
                
                if ui.button("❌ Cancel").clicked() {
                    state.import_text.clear();
                    state.show_import_dialog = false;
                }
            });
        });
}

fn save_session_dialog(state: &mut AppState) {
    let filename = format!("session_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    match state.save_session(&filename) {
        Ok(()) => state.status = format!("✅ Saved to {}", filename),
        Err(e) => state.status = format!("❌ Save failed: {}", e),
    }
}

fn load_session_dialog(state: &mut AppState) {
    if let Ok(entries) = std::fs::read_dir(".") {
        let mut session_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("session_") && name.ends_with(".json")
            })
            .collect();
        
        session_files.sort_by(|a, b| {
            b.metadata().and_then(|m| m.modified()).ok()
                .cmp(&a.metadata().and_then(|m| m.modified()).ok())
        });

        if let Some(latest) = session_files.first() {
            match state.load_session(latest.path().to_str().unwrap_or("")) {
                Ok(()) => state.status = format!("✅ Loaded {} candidates", state.candidates.len()),
                Err(e) => state.status = format!("❌ Load failed: {}", e),
            }
        } else {
            state.status = "No session files found".into();
        }
    }
}

fn export_csv(state: &mut AppState) {
    use std::io::Write;
    let filename = format!("candidates_{}.csv", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    match std::fs::File::create(&filename) {
        Ok(mut file) => {
            writeln!(file, "ID,SMILES,Efficacy,Toxicity,SynthesisCost,ManufacturingCost,Pareto,Score,Favorite").unwrap();
            for c in &state.candidates {
                let score = state.weighted_score(c);
                let fav = if state.annotations.is_favorite(c.id) { "1" } else { "0" };
                writeln!(file, "{},{},{:.4},{:.4},{:.4},{:.4},{},{:.4},{}", 
                    c.id, c.smiles, c.efficacy, c.toxicity, c.synthesis_cost, c.manufacturing_cost, c.pareto, score, fav).unwrap();
            }
            state.status = format!("✅ Exported to {}", filename);
        }
        Err(e) => state.status = format!("❌ Export failed: {}", e),
    }
}

fn export_json(state: &mut AppState) {
    use std::io::Write;
    let filename = format!("candidates_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    match std::fs::File::create(&filename) {
        Ok(mut file) => {
            let json = serde_json::to_string_pretty(&state.candidates).unwrap();
            file.write_all(json.as_bytes()).unwrap();
            state.status = format!("✅ Exported to {}", filename);
        }
        Err(e) => state.status = format!("❌ Export failed: {}", e),
    }
}

fn export_sdf(state: &mut AppState) {
    let filename = format!("candidates_{}.sdf", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    match io::export_sdf(&state.candidates, &filename) {
        Ok(()) => state.status = format!("✅ Exported to {}", filename),
        Err(e) => state.status = format!("❌ Export failed: {}", e),
    }
}

fn export_smiles(state: &mut AppState) {
    let filename = format!("candidates_{}.smi", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    match io::export_smiles_file(&state.candidates, &filename) {
        Ok(()) => state.status = format!("✅ Exported to {}", filename),
        Err(e) => state.status = format!("❌ Export failed: {}", e),
    }
}
