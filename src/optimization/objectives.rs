use crate::app::state::Candidate;
use crate::chemistry;

/// Recompute objectives for a candidate based on its SMILES.
/// This can be used when you want to recalculate scores with updated models.
pub fn compute_objectives(candidate: &mut Candidate) {
    let smiles = &candidate.smiles;
    
    // Get molecular descriptors
    let mw = chemistry::descriptors::molecular_weight_from_smiles(smiles);
    let logp = chemistry::descriptors::logp_from_smiles(smiles);
    let psa = chemistry::descriptors::polar_surface_area_from_smiles(smiles);
    let (hbd, hba) = chemistry::descriptors::hbd_hba_count(smiles);
    
    // Compute objectives
    candidate.efficacy = compute_efficacy(mw, logp, psa, hbd, hba);
    candidate.toxicity = compute_toxicity(mw, logp, psa, hbd, hba);
    candidate.synthesis_cost = compute_synthesis_cost(smiles, mw);
    candidate.manufacturing_cost = compute_manufacturing_cost(mw, logp);
}

/// Compute efficacy score based on drug-likeness criteria
fn compute_efficacy(mw: f32, logp: f32, psa: f32, hbd: usize, hba: usize) -> f32 {
    let mut score: f32 = 0.5;
    
    // Lipinski's Rule of Five compliance
    let mut violations = 0;
    if mw > 500.0 { violations += 1; }
    if logp > 5.0 { violations += 1; }
    if hbd > 5 { violations += 1; }
    if hba > 10 { violations += 1; }
    
    score += match violations {
        0 => 0.3,
        1 => 0.15,
        2 => 0.0,
        _ => -0.2,
    };
    
    // Optimal MW range (250-450 for oral drugs)
    if mw >= 250.0 && mw <= 450.0 {
        score += 0.1;
    }
    
    // Optimal logP range (1-3)
    if logp >= 1.0 && logp <= 3.0 {
        score += 0.1;
    }
    
    // PSA for CNS drugs (< 90) or general (< 140)
    if psa < 90.0 {
        score += 0.05;
    }
    
    score.clamp(0.0, 1.0)
}

/// Compute toxicity risk score
fn compute_toxicity(mw: f32, logp: f32, psa: f32, hbd: usize, hba: usize) -> f32 {
    let mut risk: f32 = 0.1;
    
    // High lipophilicity associated with toxicity
    if logp > 5.0 {
        risk += 0.3;
    } else if logp > 4.0 {
        risk += 0.15;
    }
    
    // Very large molecules
    if mw > 600.0 {
        risk += 0.2;
    }
    
    // Low PSA can indicate promiscuity
    if psa < 20.0 {
        risk += 0.15;
    }
    
    // Many H-bond sites can indicate reactivity
    if hbd > 6 || hba > 12 {
        risk += 0.1;
    }
    
    risk.clamp(0.0, 1.0)
}

/// Compute synthesis complexity/cost
fn compute_synthesis_cost(smiles: &str, mw: f32) -> f32 {
    let mut cost = 0.1;
    
    // Count complexity indicators
    let rings = smiles.chars().filter(|c| c.is_numeric()).count() / 2;
    cost += rings as f32 * 0.1;
    
    let stereo = smiles.chars().filter(|&c| c == '@' || c == '/' || c == '\\').count();
    cost += stereo as f32 * 0.15;
    
    let double_bonds = smiles.chars().filter(|&c| c == '=').count();
    cost += double_bonds as f32 * 0.03;
    
    let branches = smiles.chars().filter(|&c| c == '(').count();
    cost += branches as f32 * 0.04;
    
    // Exotic elements
    let exotic = smiles.chars().filter(|&c| "SPFClBrI".contains(c)).count();
    cost += exotic as f32 * 0.05;
    
    // Size factor
    cost += (mw / 500.0).min(0.3);
    
    cost.clamp(0.0, 1.0)
}

/// Compute manufacturing cost
fn compute_manufacturing_cost(mw: f32, logp: f32) -> f32 {
    let mut cost = 0.15;
    
    // Purification difficulty
    if logp > 4.0 {
        cost += 0.2;
    } else if logp < 0.0 {
        cost += 0.15; // Very polar, hard to handle
    }
    
    // Scale-up difficulty with size
    cost += (mw / 400.0).min(0.35);
    
    cost.clamp(0.0, 1.0)
}

/// Multi-objective weighted sum (for simple ranking)
pub fn weighted_sum(candidate: &Candidate, weights: (f32, f32, f32, f32)) -> f32 {
    let (w_eff, w_tox, w_syn, w_mfg) = weights;
    
    w_eff * candidate.efficacy
        - w_tox * candidate.toxicity
        - w_syn * candidate.synthesis_cost
        - w_mfg * candidate.manufacturing_cost
}

/// Check if candidate passes basic drug-likeness filters
pub fn passes_druglikeness_filter(candidate: &Candidate) -> bool {
    let smiles = &candidate.smiles;
    
    let mw = chemistry::descriptors::molecular_weight_from_smiles(smiles);
    let logp = chemistry::descriptors::logp_from_smiles(smiles);
    let (hbd, hba) = chemistry::descriptors::hbd_hba_count(smiles);
    
    // Extended Lipinski (Veber rules)
    mw <= 500.0 
        && logp <= 5.0 
        && hbd <= 5 
        && hba <= 10
        && candidate.toxicity < 0.7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_objectives() {
        let mut candidate = Candidate {
            id: 0,
            smiles: "CCCCCC".to_string(),
            efficacy: 0.0,
            toxicity: 0.0,
            synthesis_cost: 0.0,
            manufacturing_cost: 0.0,
            pareto: false,
        };
        
        compute_objectives(&mut candidate);
        
        assert!(candidate.efficacy > 0.0);
        assert!(candidate.synthesis_cost > 0.0);
    }
}
