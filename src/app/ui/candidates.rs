use eframe::egui;
use egui_plot::{Plot, Points, PlotPoints};
use crate::app::state::{AppState, Candidate};
use super::{visualizations, advanced_viz};

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Header
        ui.horizontal(|ui| {
            ui.heading("🧪 Candidates");
            ui.separator();
            
            let filtered = state.filtered_candidates();
            let pareto_filtered = filtered.iter().filter(|c| c.pareto).count();
            
            ui.label(format!("Showing: {} | Pareto: {} | Total: {}", 
                filtered.len(), pareto_filtered, state.candidates.len()));
        });

        visualizations::render_stats_summary(ui, state);
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Scatter plots
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("📈 Efficacy vs Toxicity");
                        render_scatter_plot(ui, state, "eff_vs_tox", 
                            |c| c.toxicity, |c| c.efficacy, "Toxicity", "Efficacy");
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label("📈 Costs");
                        render_scatter_plot(ui, state, "costs",
                            |c| c.synthesis_cost, |c| c.manufacturing_cost, "Synth", "Mfg");
                    });
                });

                ui.separator();

                // Advanced visualizations
                if state.show_3d_plot {
                    ui.collapsing("🎲 3D Plot", |ui| {
                        advanced_viz::render_3d_plot(ui, state);
                    });
                }

                if state.show_heatmap {
                    ui.collapsing("🔥 Correlation Heatmap", |ui| {
                        advanced_viz::render_correlation_heatmap(ui, state);
                    });
                }

                if state.show_histograms {
                    ui.collapsing("📊 Histograms", |ui| {
                        visualizations::render_histograms(ui, state);
                    });
                }

                if state.show_parallel_coords {
                    ui.collapsing("📈 Parallel Coordinates", |ui| {
                        visualizations::render_parallel_coordinates(ui, state);
                    });
                }

                if state.show_clustering {
                    ui.collapsing("🔬 Clustering", |ui| {
                        advanced_viz::render_clustering_view(ui, state);
                    });
                }

                if state.show_similarity_search {
                    ui.collapsing("🔍 Similarity Search", |ui| {
                        advanced_viz::render_similarity_search(ui, state);
                    });
                }

                ui.separator();
                ui.label("📋 Table");

                // Table
                let mut rows: Vec<Candidate> = state.filtered_candidates()
                    .into_iter()
                    .cloned()
                    .collect();

                rows.sort_by(|a, b| {
                    state.weighted_score(b)
                        .partial_cmp(&state.weighted_score(a))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                render_table(ui, state, &rows);
            });
    });
}

fn render_scatter_plot<F1, F2>(
    ui: &mut egui::Ui,
    state: &AppState,
    id: &str,
    x_fn: F1,
    y_fn: F2,
    x_label: &str,
    y_label: &str,
) where
    F1: Fn(&Candidate) -> f32,
    F2: Fn(&Candidate) -> f32,
{
    let filtered = state.filtered_candidates();

    let pareto_points: PlotPoints = filtered.iter()
        .filter(|c| c.pareto)
        .map(|c| [x_fn(c) as f64, y_fn(c) as f64])
        .collect();

    let non_pareto_points: PlotPoints = filtered.iter()
        .filter(|c| !c.pareto)
        .map(|c| [x_fn(c) as f64, y_fn(c) as f64])
        .collect();

    let favorite_points: PlotPoints = filtered.iter()
        .filter(|c| state.annotations.is_favorite(c.id))
        .map(|c| [x_fn(c) as f64, y_fn(c) as f64])
        .collect();

    let selected_points: PlotPoints = if let Some(id) = state.selected_id {
        filtered.iter()
            .filter(|c| c.id == id)
            .map(|c| [x_fn(c) as f64, y_fn(c) as f64])
            .collect()
    } else {
        PlotPoints::new(vec![])
    };

    Plot::new(id)
        .view_aspect(1.3)
        .height(180.0)
        .x_axis_label(x_label)
        .y_axis_label(y_label)
        .show(ui, |plot_ui| {
            plot_ui.points(Points::new(non_pareto_points).name("Regular").color(egui::Color32::from_rgb(150, 150, 150)).radius(3.0));
            plot_ui.points(Points::new(pareto_points).name("Pareto").color(egui::Color32::from_rgb(0, 200, 100)).radius(5.0));
            plot_ui.points(Points::new(favorite_points).name("Favorite").color(egui::Color32::from_rgb(255, 200, 50)).radius(6.0));
            plot_ui.points(Points::new(selected_points).name("Selected").color(egui::Color32::from_rgb(255, 100, 100)).radius(8.0));
        });
}

fn render_table(ui: &mut egui::Ui, state: &mut AppState, rows: &[Candidate]) {
    egui::Grid::new("candidates_grid")
        .striped(true)
        .min_col_width(40.0)
        .show(ui, |ui| {
            // Header
            ui.strong("");
            ui.strong("⭐");
            ui.strong("ID");
            ui.strong("SMILES");
            ui.strong("Eff");
            ui.strong("Tox");
            ui.strong("Syn");
            ui.strong("Mfg");
            ui.strong("Score");
            ui.strong("P");
            ui.end_row();

            for c in rows.iter().take(1500) {
                let selected = state.selected_id == Some(c.id);
                let is_fav = state.annotations.is_favorite(c.id);
                
                if ui.selectable_label(selected, if selected { "▶" } else { "○" }).clicked() {
                    state.selected_id = Some(c.id);
                }
                
                // Favorite
                let fav_text = if is_fav { "⭐" } else { "" };
                ui.label(fav_text);
                
                ui.label(c.id.to_string());
                
                let smiles_display = if c.smiles.len() > 20 {
                    format!("{}...", &c.smiles[..20])
                } else {
                    c.smiles.clone()
                };
                if ui.monospace(smiles_display).on_hover_text(&c.smiles).clicked() {
                    state.selected_id = Some(c.id);
                }
                
                ui.colored_label(color_for_value(c.efficacy, true), format!("{:.3}", c.efficacy));
                ui.colored_label(color_for_value(c.toxicity, false), format!("{:.3}", c.toxicity));
                ui.label(format!("{:.3}", c.synthesis_cost));
                ui.label(format!("{:.3}", c.manufacturing_cost));
                
                let score = state.weighted_score(c);
                ui.colored_label(color_for_score(score), format!("{:.3}", score));
                
                if c.pareto { ui.colored_label(egui::Color32::from_rgb(0, 200, 100), "✓"); } else { ui.label(""); }
                
                ui.end_row();
            }
        });
    
    if rows.len() > 1500 {
        ui.label(format!("... +{} more", rows.len() - 1500));
    }
}

fn color_for_value(value: f32, higher_is_better: bool) -> egui::Color32 {
    let normalized = value.clamp(0.0, 1.0);
    let good = if higher_is_better { normalized } else { 1.0 - normalized };
    egui::Color32::from_rgb(((1.0 - good) * 255.0) as u8, (good * 200.0) as u8, 80)
}

fn color_for_score(score: f32) -> egui::Color32 {
    let normalized = ((score + 2.0) / 4.0).clamp(0.0, 1.0);
    egui::Color32::from_rgb(((1.0 - normalized) * 200.0) as u8, (normalized * 200.0) as u8, 80)
}
