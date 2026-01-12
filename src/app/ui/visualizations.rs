//! Advanced visualizations: histograms and parallel coordinates

use eframe::egui;
use egui_plot::{Plot, Bar, BarChart, Line, PlotPoints};
use crate::app::state::{AppState, Candidate};

/// Render histograms for all objectives
pub fn render_histograms(ui: &mut egui::Ui, state: &AppState) {
    let candidates = state.filtered_candidates();
    
    if candidates.is_empty() {
        ui.label("No candidates to display");
        return;
    }

    ui.horizontal(|ui| {
        // Efficacy histogram
        ui.vertical(|ui| {
            ui.label("📊 Efficacy Distribution");
            render_histogram(ui, "hist_eff", &candidates, |c| c.efficacy, egui::Color32::from_rgb(100, 200, 100));
        });

        ui.separator();

        // Toxicity histogram
        ui.vertical(|ui| {
            ui.label("📊 Toxicity Distribution");
            render_histogram(ui, "hist_tox", &candidates, |c| c.toxicity, egui::Color32::from_rgb(255, 150, 100));
        });
    });

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        // Synthesis cost histogram
        ui.vertical(|ui| {
            ui.label("📊 Synthesis Cost Distribution");
            render_histogram(ui, "hist_syn", &candidates, |c| c.synthesis_cost, egui::Color32::from_rgb(100, 150, 255));
        });

        ui.separator();

        // Manufacturing cost histogram
        ui.vertical(|ui| {
            ui.label("📊 Manufacturing Cost Distribution");
            render_histogram(ui, "hist_mfg", &candidates, |c| c.manufacturing_cost, egui::Color32::from_rgb(200, 100, 200));
        });
    });
}

fn render_histogram<F>(
    ui: &mut egui::Ui,
    id: &str,
    candidates: &[&Candidate],
    value_fn: F,
    color: egui::Color32,
) where
    F: Fn(&Candidate) -> f32,
{
    let num_bins = 20;
    let mut bins = vec![0u32; num_bins];
    
    // Calculate histogram
    for c in candidates {
        let value = value_fn(c).clamp(0.0, 1.0);
        let bin = ((value * num_bins as f32) as usize).min(num_bins - 1);
        bins[bin] += 1;
    }

    // Convert to bars
    let bars: Vec<Bar> = bins
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            let x = (i as f64 + 0.5) / num_bins as f64;
            Bar::new(x, count as f64)
                .width(0.8 / num_bins as f64)
                .fill(color)
        })
        .collect();

    let chart = BarChart::new(bars);

    Plot::new(id)
        .height(120.0)
        .width(200.0)
        .show_axes([true, true])
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
        });
}

/// Render parallel coordinates plot
pub fn render_parallel_coordinates(ui: &mut egui::Ui, state: &AppState) {
    let candidates = state.filtered_candidates();
    
    if candidates.is_empty() {
        ui.label("No candidates to display");
        return;
    }

    ui.label("📈 Parallel Coordinates (normalized 0-1)");
    ui.small("Each line represents one candidate. Pareto optimal = green, others = gray");

    let plot_height = 250.0;

    Plot::new("parallel_coords")
        .height(plot_height)
        .show_axes([true, true])
        .x_axis_label("Objectives")
        .y_axis_label("Value (normalized)")
        .show(ui, |plot_ui| {
            // Draw axis labels
            let axis_positions = [0.0, 1.0, 2.0, 3.0];
            
            // Draw each candidate as a line
            // Limit to 500 for performance
            let max_display = 500;
            let step = if candidates.len() > max_display {
                candidates.len() / max_display
            } else {
                1
            };

            for (i, &c) in candidates.iter().enumerate() {
                if i % step != 0 {
                    continue;
                }

                // Normalize values and invert toxicity/costs (lower is better)
                let values = [
                    c.efficacy as f64,                    // Higher is better
                    1.0 - c.toxicity as f64,              // Invert: lower tox = higher value
                    1.0 - c.synthesis_cost as f64,        // Invert
                    1.0 - c.manufacturing_cost as f64,    // Invert
                ];

                let points: PlotPoints = axis_positions
                    .iter()
                    .zip(values.iter())
                    .map(|(&x, &y)| [x, y])
                    .collect();

                let color = if c.pareto {
                    egui::Color32::from_rgba_unmultiplied(0, 200, 100, 200)
                } else {
                    egui::Color32::from_rgba_unmultiplied(150, 150, 150, 50)
                };

                let line = Line::new(points)
                    .color(color)
                    .width(if c.pareto { 2.0 } else { 1.0 });
                
                plot_ui.line(line);
            }

            // Draw vertical axis lines
            for &x in &axis_positions {
                let axis_line = Line::new(PlotPoints::new(vec![[x, 0.0], [x, 1.0]]))
                    .color(egui::Color32::from_rgb(100, 100, 100))
                    .width(1.0);
                plot_ui.line(axis_line);
            }
        });

    // Legend
    ui.horizontal(|ui| {
        ui.label("Axes: ");
        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "0=Efficacy");
        ui.label("|");
        ui.colored_label(egui::Color32::from_rgb(255, 150, 100), "1=1-Toxicity");
        ui.label("|");
        ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "2=1-SynthCost");
        ui.label("|");
        ui.colored_label(egui::Color32::from_rgb(200, 100, 200), "3=1-MfgCost");
    });
}

/// Render a compact stats summary
pub fn render_stats_summary(ui: &mut egui::Ui, state: &AppState) {
    let candidates = state.filtered_candidates();
    
    if candidates.is_empty() {
        return;
    }

    let n = candidates.len() as f32;
    
    // Calculate stats
    let avg_eff: f32 = candidates.iter().map(|c| c.efficacy).sum::<f32>() / n;
    let avg_tox: f32 = candidates.iter().map(|c| c.toxicity).sum::<f32>() / n;
    let avg_syn: f32 = candidates.iter().map(|c| c.synthesis_cost).sum::<f32>() / n;
    let avg_mfg: f32 = candidates.iter().map(|c| c.manufacturing_cost).sum::<f32>() / n;

    let max_eff = candidates.iter().map(|c| c.efficacy).fold(0.0f32, f32::max);
    let min_tox = candidates.iter().map(|c| c.toxicity).fold(1.0f32, f32::min);

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(format!("📊 Filtered: {} |", candidates.len()));
            ui.colored_label(
                egui::Color32::from_rgb(100, 200, 100),
                format!("Eff: avg={:.3} max={:.3}", avg_eff, max_eff)
            );
            ui.label("|");
            ui.colored_label(
                egui::Color32::from_rgb(255, 150, 100),
                format!("Tox: avg={:.3} min={:.3}", avg_tox, min_tox)
            );
            ui.label("|");
            ui.label(format!("Syn: {:.3} | Mfg: {:.3}", avg_syn, avg_mfg));
        });
    });
}

/// Calculate and display correlation matrix
pub fn render_correlation_hint(ui: &mut egui::Ui, state: &AppState) {
    let candidates = state.filtered_candidates();
    
    if candidates.len() < 10 {
        return;
    }

    // Simple correlation between efficacy and toxicity
    let n = candidates.len() as f32;
    let mean_eff: f32 = candidates.iter().map(|c| c.efficacy).sum::<f32>() / n;
    let mean_tox: f32 = candidates.iter().map(|c| c.toxicity).sum::<f32>() / n;

    let mut cov = 0.0f32;
    let mut var_eff = 0.0f32;
    let mut var_tox = 0.0f32;

    for c in &candidates {
        let d_eff = c.efficacy - mean_eff;
        let d_tox = c.toxicity - mean_tox;
        cov += d_eff * d_tox;
        var_eff += d_eff * d_eff;
        var_tox += d_tox * d_tox;
    }

    let corr = if var_eff > 0.0 && var_tox > 0.0 {
        cov / (var_eff.sqrt() * var_tox.sqrt())
    } else {
        0.0
    };

    let color = if corr < -0.3 {
        egui::Color32::from_rgb(100, 200, 100) // Good: negative correlation
    } else if corr > 0.3 {
        egui::Color32::from_rgb(255, 100, 100) // Bad: positive correlation
    } else {
        egui::Color32::from_rgb(200, 200, 200) // Neutral
    };

    ui.horizontal(|ui| {
        ui.label("Eff-Tox correlation:");
        ui.colored_label(color, format!("{:.3}", corr));
        if corr < -0.3 {
            ui.small("(good: high eff tends to have low tox)");
        } else if corr > 0.3 {
            ui.small("(warning: high eff tends to have high tox)");
        }
    });
}
