use eframe::egui;
use crate::app::state::AppState;
use super::advanced_viz;

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    egui::SidePanel::left("side_panel")
        .resizable(true)
        .min_width(280.0)
        .default_width(320.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("⚙️ Controls");
                ui.add_space(5.0);

                // Filters
                ui.collapsing("🔍 Filters", |ui| {
                    ui.checkbox(&mut state.filter_pareto_only, "Pareto front only");
                    ui.checkbox(&mut state.filter_favorites_only, "⭐ Favorites only");
                    
                    ui.add_space(5.0);
                    ui.label("SMILES search:");
                    ui.text_edit_singleline(&mut state.filter_smiles);
                    
                    ui.add_space(5.0);
                    ui.label("Efficacy:");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut state.filter_eff_min).clamp_range(0.0..=1.0).speed(0.01).prefix("min: "));
                        ui.add(egui::DragValue::new(&mut state.filter_eff_max).clamp_range(0.0..=1.0).speed(0.01).prefix("max: "));
                    });
                    
                    ui.label("Toxicity:");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut state.filter_tox_min).clamp_range(0.0..=1.0).speed(0.01).prefix("min: "));
                        ui.add(egui::DragValue::new(&mut state.filter_tox_max).clamp_range(0.0..=1.0).speed(0.01).prefix("max: "));
                    });

                    if ui.button("Reset Filters").clicked() {
                        state.filter_smiles.clear();
                        state.filter_eff_min = 0.0;
                        state.filter_eff_max = 1.0;
                        state.filter_tox_min = 0.0;
                        state.filter_tox_max = 1.0;
                        state.filter_pareto_only = false;
                        state.filter_favorites_only = false;
                    }
                });

                ui.add_space(5.0);

                // Weights
                ui.collapsing("⚖️ Weights", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Efficacy (+):");
                        ui.add(egui::Slider::new(&mut state.w_eff, 0.0..=5.0).step_by(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Toxicity (-):");
                        ui.add(egui::Slider::new(&mut state.w_tox, 0.0..=5.0).step_by(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Synthesis (-):");
                        ui.add(egui::Slider::new(&mut state.w_syn, 0.0..=5.0).step_by(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mfg (-):");
                        ui.add(egui::Slider::new(&mut state.w_mfg, 0.0..=5.0).step_by(0.1));
                    });
                    if ui.button("Reset").clicked() {
                        state.w_eff = 1.0;
                        state.w_tox = 1.0;
                        state.w_syn = 1.0;
                        state.w_mfg = 1.0;
                    }
                });

                ui.add_space(5.0);

                // Statistics
                ui.collapsing("📊 Statistics", |ui| {
                    let total = state.candidates.len();
                    let filtered = state.filtered_candidates().len();
                    let pareto = state.candidates.iter().filter(|c| c.pareto).count();
                    let favorites = state.annotations.favorite_count();
                    
                    ui.label(format!("Total: {} | Filtered: {}", total, filtered));
                    ui.label(format!("Pareto: {} | ⭐ Favorites: {}", pareto, favorites));
                    
                    if total > 0 {
                        let avg_eff: f32 = state.candidates.iter().map(|c| c.efficacy).sum::<f32>() / total as f32;
                        let avg_tox: f32 = state.candidates.iter().map(|c| c.toxicity).sum::<f32>() / total as f32;
                        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), format!("Avg Eff: {:.3}", avg_eff));
                        ui.colored_label(egui::Color32::from_rgb(255, 150, 100), format!("Avg Tox: {:.3}", avg_tox));
                    }
                    
                    ui.label(format!("History: {} undo, {} redo", state.history.undo_count(), state.history.redo_count()));
                });

                ui.add_space(5.0);

                // Drug-likeness panel
                if state.show_druglikeness {
                    ui.collapsing("💊 Drug-likeness", |ui| {
                        advanced_viz::render_druglikeness_panel(ui, state);
                    });
                }

                ui.add_space(5.0);

                // Selected candidate
                ui.group(|ui| {
                    ui.label("📋 Selected");
                    
                    if let Some(id) = state.selected_id {
                        // Copy candidate data to avoid borrow issues
                        let candidate_data = state.candidates.iter().find(|x| x.id == id).cloned();
                        let is_fav = state.annotations.is_favorite(id);
                        let note_text = state.annotations.get_note(id).cloned().unwrap_or_default();
                        
                        if let Some(c) = candidate_data {
                            let score = state.weighted_score(&c);
                            
                            ui.horizontal(|ui| {
                                ui.label(format!("ID: {}", c.id));
                                
                                let fav_btn = if is_fav { "⭐" } else { "☆" };
                                if ui.button(fav_btn).on_hover_text("Toggle favorite").clicked() {
                                    state.toggle_favorite(c.id);
                                }
                            });
                            
                            ui.label("SMILES:");
                            ui.horizontal_wrapped(|ui| {
                                ui.monospace(&c.smiles);
                            });
                            
                            if ui.small_button("📋 Copy").clicked() {
                                ui.output_mut(|o| o.copied_text = c.smiles.clone());
                            }
                            
                            ui.separator();
                            
                            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), format!("Efficacy: {:.4}", c.efficacy));
                            ui.colored_label(egui::Color32::from_rgb(255, 150, 100), format!("Toxicity: {:.4}", c.toxicity));
                            ui.label(format!("Synth: {:.4}", c.synthesis_cost));
                            ui.label(format!("Mfg: {:.4}", c.manufacturing_cost));
                            ui.strong(format!("Score: {:.4}", score));
                            
                            if c.pareto {
                                ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "✅ Pareto optimal");
                            }
                            
                            // Annotation
                            ui.separator();
                            ui.label("📝 Note:");
                            let mut note = note_text;
                            if ui.text_edit_multiline(&mut note).changed() {
                                state.set_note(c.id, note);
                            }
                        }
                    } else {
                        ui.label("Click to select");
                    }
                });

                // Footer
                ui.add_space(10.0);
                ui.separator();
                ui.small("Drug Candidate Studio v0.3.0");
                ui.small(format!("CPU: {} cores", rayon::current_num_threads()));
            });
        });
}
