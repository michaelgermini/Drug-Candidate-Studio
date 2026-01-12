// Additional Pareto visualization utilities
// Can be extended for 3D views, parallel coordinates, etc.

use crate::app::state::Candidate;

/// Calculate the hypervolume indicator for a set of candidates
/// This is a common metric in multi-objective optimization
pub fn hypervolume_2d(candidates: &[Candidate], ref_point: (f32, f32)) -> f32 {
    // Sort candidates by first objective (efficacy, descending)
    let mut pareto: Vec<_> = candidates.iter().filter(|c| c.pareto).collect();
    pareto.sort_by(|a, b| b.efficacy.partial_cmp(&a.efficacy).unwrap());
    
    let mut hv = 0.0;
    let mut prev_tox = 0.0;
    
    for c in pareto {
        if c.toxicity > ref_point.1 || c.efficacy < ref_point.0 {
            continue;
        }
        
        let width = c.toxicity - prev_tox;
        let height = c.efficacy - ref_point.0;
        
        if width > 0.0 && height > 0.0 {
            hv += width * height;
        }
        
        prev_tox = c.toxicity;
    }
    
    hv
}

/// Find the "knee point" of the Pareto front
/// The knee is the point with maximum distance to the line connecting extremes
pub fn find_knee_point(candidates: &[Candidate]) -> Option<usize> {
    let pareto: Vec<_> = candidates.iter().filter(|c| c.pareto).collect();
    
    if pareto.len() < 3 {
        return pareto.first().map(|c| c.id);
    }
    
    // Find extremes
    let max_eff = pareto.iter().max_by(|a, b| a.efficacy.partial_cmp(&b.efficacy).unwrap())?;
    let min_tox = pareto.iter().min_by(|a, b| a.toxicity.partial_cmp(&b.toxicity).unwrap())?;
    
    // Line from max_eff to min_tox
    let (x1, y1) = (max_eff.toxicity, max_eff.efficacy);
    let (x2, y2) = (min_tox.toxicity, min_tox.efficacy);
    
    // Find point with maximum distance to line
    let mut max_dist = 0.0;
    let mut knee_id = None;
    
    for c in &pareto {
        let dist = point_to_line_distance(c.toxicity, c.efficacy, x1, y1, x2, y2);
        if dist > max_dist {
            max_dist = dist;
            knee_id = Some(c.id);
        }
    }
    
    knee_id
}

fn point_to_line_distance(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let num = ((y2 - y1) * px - (x2 - x1) * py + x2 * y1 - y2 * x1).abs();
    let den = ((y2 - y1).powi(2) + (x2 - x1).powi(2)).sqrt();
    
    if den > 0.0 {
        num / den
    } else {
        0.0
    }
}
