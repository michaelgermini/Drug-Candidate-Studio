//! Advanced visualizations: 3D plot, heatmap, clustering view

use eframe::egui;
use egui_plot::{Plot, Points, PlotPoints, Line, BarChart, Bar};
use crate::app::state::{AppState, Candidate};
use crate::chemistry::similarity;

/// Render 3D-like scatter plot using perspective projection
pub fn render_3d_plot(ui: &mut egui::Ui, state: &AppState) {
    let candidates = state.filtered_candidates();
    
    if candidates.is_empty() {
        ui.label("No candidates to display");
        return;
    }

    ui.label("🎲 3D View: Efficacy × Toxicity × Synthesis Cost");
    ui.small("Rotate with angle slider. Size = Manufacturing cost (smaller = better)");

    // Rotation angle control
    static mut ROTATION_ANGLE: f32 = 0.3;
    let angle = unsafe { &mut ROTATION_ANGLE };
    
    ui.horizontal(|ui| {
        ui.label("Rotation:");
        ui.add(egui::Slider::new(angle, 0.0..=std::f32::consts::TAU).text("angle"));
    });

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    Plot::new("3d_plot")
        .height(300.0)
        .data_aspect(1.0)
        .x_axis_label("X (rotated)")
        .y_axis_label("Efficacy")
        .show(ui, |plot_ui| {
            // Project 3D points to 2D with rotation
            let pareto_points: PlotPoints = candidates
                .iter()
                .filter(|c| c.pareto)
                .map(|c| {
                    // 3D coordinates: x=toxicity, y=efficacy, z=synthesis_cost
                    let x = c.toxicity as f64;
                    let z = c.synthesis_cost as f64;
                    // Apply rotation around Y axis
                    let x_rot = x * cos_a as f64 + z * sin_a as f64;
                    let y = c.efficacy as f64;
                    [x_rot, y]
                })
                .collect();

            let non_pareto_points: PlotPoints = candidates
                .iter()
                .filter(|c| !c.pareto)
                .map(|c| {
                    let x = c.toxicity as f64;
                    let z = c.synthesis_cost as f64;
                    let x_rot = x * cos_a as f64 + z * sin_a as f64;
                    let y = c.efficacy as f64;
                    [x_rot, y]
                })
                .collect();

            plot_ui.points(
                Points::new(non_pareto_points)
                    .name("Non-Pareto")
                    .color(egui::Color32::from_rgba_unmultiplied(150, 150, 150, 100))
                    .radius(2.0)
            );
            
            plot_ui.points(
                Points::new(pareto_points)
                    .name("Pareto")
                    .color(egui::Color32::from_rgb(0, 220, 100))
                    .radius(5.0)
            );
        });
}

/// Render correlation heatmap between objectives
pub fn render_correlation_heatmap(ui: &mut egui::Ui, state: &AppState) {
    let candidates = state.filtered_candidates();
    
    if candidates.len() < 10 {
        ui.label("Need at least 10 candidates for correlation analysis");
        return;
    }

    ui.label("🔥 Correlation Heatmap");
    ui.small("Shows Pearson correlation between objectives (-1 to +1)");

    // Calculate correlations
    let objectives: Vec<(&str, Box<dyn Fn(&Candidate) -> f32>)> = vec![
        ("Efficacy", Box::new(|c: &Candidate| c.efficacy)),
        ("Toxicity", Box::new(|c: &Candidate| c.toxicity)),
        ("SynthCost", Box::new(|c: &Candidate| c.synthesis_cost)),
        ("MfgCost", Box::new(|c: &Candidate| c.manufacturing_cost)),
    ];

    let n = objectives.len();
    let mut correlations = vec![vec![0.0f32; n]; n];

    for i in 0..n {
        for j in 0..n {
            if i == j {
                correlations[i][j] = 1.0;
            } else if j > i {
                let corr = calculate_correlation(&candidates, &objectives[i].1, &objectives[j].1);
                correlations[i][j] = corr;
                correlations[j][i] = corr;
            }
        }
    }

    // Draw heatmap as a grid
    let cell_size = 50.0;
    
    egui::Grid::new("heatmap")
        .spacing([2.0, 2.0])
        .show(ui, |ui| {
            // Header row
            ui.label("");
            for (name, _) in &objectives {
                ui.label(*name);
            }
            ui.end_row();

            // Data rows
            for i in 0..n {
                ui.label(objectives[i].0);
                for j in 0..n {
                    let corr = correlations[i][j];
                    let color = correlation_color(corr);
                    
                    let (rect, _response) = ui.allocate_exact_size(
                        egui::vec2(cell_size, 25.0),
                        egui::Sense::hover()
                    );
                    
                    ui.painter().rect_filled(rect, 3.0, color);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{:.2}", corr),
                        egui::FontId::default(),
                        if corr.abs() > 0.5 { egui::Color32::WHITE } else { egui::Color32::BLACK }
                    );
                }
                ui.end_row();
            }
        });

    // Legend
    ui.horizontal(|ui| {
        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "🔴 Negative");
        ui.label("|");
        ui.colored_label(egui::Color32::from_rgb(200, 200, 200), "⚪ Zero");
        ui.label("|");
        ui.colored_label(egui::Color32::from_rgb(100, 100, 255), "🔵 Positive");
    });
}

fn calculate_correlation<F1, F2>(candidates: &[&Candidate], f1: &F1, f2: &F2) -> f32
where
    F1: Fn(&Candidate) -> f32,
    F2: Fn(&Candidate) -> f32,
{
    let n = candidates.len() as f32;
    if n < 2.0 {
        return 0.0;
    }

    let mean1: f32 = candidates.iter().map(|c| f1(c)).sum::<f32>() / n;
    let mean2: f32 = candidates.iter().map(|c| f2(c)).sum::<f32>() / n;

    let mut cov = 0.0f32;
    let mut var1 = 0.0f32;
    let mut var2 = 0.0f32;

    for c in candidates {
        let d1 = f1(c) - mean1;
        let d2 = f2(c) - mean2;
        cov += d1 * d2;
        var1 += d1 * d1;
        var2 += d2 * d2;
    }

    if var1 > 0.0 && var2 > 0.0 {
        cov / (var1.sqrt() * var2.sqrt())
    } else {
        0.0
    }
}

fn correlation_color(corr: f32) -> egui::Color32 {
    let intensity = corr.abs();
    if corr < 0.0 {
        // Negative: red
        egui::Color32::from_rgb(
            (150.0 + 105.0 * intensity) as u8,
            (150.0 - 100.0 * intensity) as u8,
            (150.0 - 100.0 * intensity) as u8,
        )
    } else {
        // Positive: blue
        egui::Color32::from_rgb(
            (150.0 - 100.0 * intensity) as u8,
            (150.0 - 100.0 * intensity) as u8,
            (150.0 + 105.0 * intensity) as u8,
        )
    }
}

/// Render clustering view
pub fn render_clustering_view(ui: &mut egui::Ui, state: &mut AppState) {
    // Copy data to avoid borrow issues
    let candidates_data: Vec<(usize, String, bool)> = state.filtered_candidates()
        .iter()
        .map(|c| (c.id, c.smiles.clone(), c.pareto))
        .collect();
    
    if candidates_data.len() < 5 {
        ui.label("Need at least 5 candidates for clustering");
        return;
    }

    ui.label("🔬 Molecular Clustering (Tanimoto similarity)");
    
    static mut CLUSTER_THRESHOLD: f32 = 0.5;
    let threshold = unsafe { &mut CLUSTER_THRESHOLD };
    
    ui.horizontal(|ui| {
        ui.label("Similarity threshold:");
        ui.add(egui::Slider::new(threshold, 0.2..=0.9).step_by(0.05));
    });

    let max_cluster = 200.min(candidates_data.len());
    let smiles_list: Vec<String> = candidates_data[..max_cluster]
        .iter()
        .map(|(_, s, _)| s.clone())
        .collect();
    
    let clusters = similarity::cluster_molecules(&smiles_list, *threshold);

    ui.separator();
    ui.label(format!("Found {} clusters from {} molecules", clusters.len(), max_cluster));
    
    // Collect click actions
    let mut click_id: Option<usize> = None;
    
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for cluster in &clusters {
                let header = format!("Cluster {} ({} members)", cluster.cluster_id, cluster.members.len());
                
                ui.collapsing(header, |ui| {
                    if let Some(&centroid_local) = cluster.members.first() {
                        if centroid_local < candidates_data.len() {
                            ui.horizontal(|ui| {
                                ui.label("Centroid:");
                                ui.monospace(&candidates_data[centroid_local].1);
                            });
                        }
                    }
                    
                    let pareto_count = cluster.members.iter()
                        .filter(|&&i| i < candidates_data.len() && candidates_data[i].2)
                        .count();
                    ui.label(format!("Pareto: {}", pareto_count));
                    
                    ui.horizontal_wrapped(|ui| {
                        for &member_idx in cluster.members.iter().take(10) {
                            if member_idx < candidates_data.len() {
                                let (id, _, pareto) = &candidates_data[member_idx];
                                let label = if *pareto { format!("✅{}", id) } else { format!("{}", id) };
                                if ui.small_button(&label).clicked() {
                                    click_id = Some(*id);
                                }
                            }
                        }
                        if cluster.members.len() > 10 {
                            ui.label(format!("... +{}", cluster.members.len() - 10));
                        }
                    });
                });
            }
        });

    // Apply click action
    if let Some(id) = click_id {
        state.selected_id = Some(id);
    }

    if candidates_data.len() >= 10 {
        let sample_smiles: Vec<String> = candidates_data[..10].iter().map(|(_, s, _)| s.clone()).collect();
        let diversity = similarity::calculate_diversity(&sample_smiles);
        ui.separator();
        ui.label(format!("Diversity: {:.3}", diversity));
    }
}

/// Render similarity search
pub fn render_similarity_search(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label("🔍 Similarity Search");
    
    static mut QUERY_SMILES: String = String::new();
    let query = unsafe { &mut QUERY_SMILES };
    
    ui.horizontal(|ui| {
        ui.label("Query SMILES:");
        ui.text_edit_singleline(query);
        
        if ui.button("Search").clicked() && !query.is_empty() {
            // Search will happen below
        }
    });

    if !query.is_empty() && state.candidates.len() > 0 {
        let smiles_list: Vec<String> = state.candidates.iter().map(|c| c.smiles.clone()).collect();
        let similar = similarity::find_similar(query, &smiles_list, 10);
        
        if !similar.is_empty() {
            ui.separator();
            ui.label("Most similar candidates:");
            
            egui::Grid::new("similar_results")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Rank");
                    ui.strong("ID");
                    ui.strong("Similarity");
                    ui.strong("SMILES");
                    ui.end_row();
                    
                    for (rank, (idx, sim)) in similar.iter().enumerate() {
                        if *idx < state.candidates.len() {
                            let c = &state.candidates[*idx];
                            ui.label(format!("{}", rank + 1));
                            
                            if ui.button(format!("{}", c.id)).clicked() {
                                state.selected_id = Some(c.id);
                            }
                            
                            ui.colored_label(
                                similarity_color(*sim),
                                format!("{:.3}", sim)
                            );
                            
                            let display = if c.smiles.len() > 30 {
                                format!("{}...", &c.smiles[..30])
                            } else {
                                c.smiles.clone()
                            };
                            ui.monospace(display);
                            ui.end_row();
                        }
                    }
                });
        }
    }
}

fn similarity_color(sim: f32) -> egui::Color32 {
    let g = (sim * 200.0) as u8;
    let r = ((1.0 - sim) * 200.0) as u8;
    egui::Color32::from_rgb(r, g, 100)
}

/// Render drug-likeness analysis panel
pub fn render_druglikeness_panel(ui: &mut egui::Ui, state: &AppState) {
    use crate::chemistry::druglikeness;
    
    ui.label("💊 Drug-likeness Analysis");
    
    if let Some(id) = state.selected_id {
        if let Some(c) = state.candidates.iter().find(|x| x.id == id) {
            let result = druglikeness::assess_druglikeness(&c.smiles);
            
            // Overall score
            ui.horizontal(|ui| {
                ui.label("Overall score:");
                let color = if result.overall_score >= 0.7 {
                    egui::Color32::from_rgb(100, 200, 100)
                } else if result.overall_score >= 0.4 {
                    egui::Color32::from_rgb(255, 200, 100)
                } else {
                    egui::Color32::from_rgb(255, 100, 100)
                };
                ui.colored_label(color, format!("{:.2}", result.overall_score));
            });
            
            ui.label(&result.recommendation);
            
            ui.separator();
            
            // Lipinski
            ui.collapsing("Lipinski's Rule of Five", |ui| {
                let lip = &result.lipinski;
                ui.horizontal(|ui| {
                    ui.label(if lip.mw_ok { "✅" } else { "❌" });
                    ui.label("MW ≤ 500");
                });
                ui.horizontal(|ui| {
                    ui.label(if lip.logp_ok { "✅" } else { "❌" });
                    ui.label("LogP ≤ 5");
                });
                ui.horizontal(|ui| {
                    ui.label(if lip.hbd_ok { "✅" } else { "❌" });
                    ui.label("H-bond donors ≤ 5");
                });
                ui.horizontal(|ui| {
                    ui.label(if lip.hba_ok { "✅" } else { "❌" });
                    ui.label("H-bond acceptors ≤ 10");
                });
                ui.label(format!("Violations: {}", lip.violations));
            });
            
            // Veber
            ui.collapsing("Veber Rules", |ui| {
                let veb = &result.veber;
                ui.horizontal(|ui| {
                    ui.label(if veb.rotatable_bonds_ok { "✅" } else { "❌" });
                    ui.label("Rotatable bonds ≤ 10");
                });
                ui.horizontal(|ui| {
                    ui.label(if veb.psa_ok { "✅" } else { "❌" });
                    ui.label("PSA ≤ 140 Ų");
                });
            });
            
            // PAINS alerts
            if !result.pains_alerts.is_empty() {
                ui.collapsing(format!("⚠️ PAINS Alerts ({})", result.pains_alerts.len()), |ui| {
                    for alert in &result.pains_alerts {
                        ui.colored_label(egui::Color32::from_rgb(255, 150, 100), alert);
                    }
                });
            } else {
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "✅ No PAINS alerts");
            }
        }
    } else {
        ui.label("Select a candidate to analyze");
    }
}
