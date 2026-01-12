// Calculs de propriétés moléculaires à partir de SMILES
use std::collections::HashMap;

/// Calculate molecular weight from SMILES string
pub fn molecular_weight_from_smiles(smiles: &str) -> f32 {
    let atomic_masses = get_atomic_masses();
    let mut total_mass = 0.0;

    let chars: Vec<char> = smiles.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let c = chars[i];

        if c.is_uppercase() {
            // Atom (potentially 2 letters like Cl, Br)
            let mut atom = c.to_string();
            i += 1;

            // Check if there's a following lowercase letter
            if i < chars.len() {
                let next_c = chars[i];
                if next_c.is_lowercase() {
                    atom.push(next_c);
                    i += 1;
                }
            }

            // Add atomic mass
            if let Some(&mass) = atomic_masses.get(&atom) {
                total_mass += mass;
            }
        } else {
            i += 1;
        }
    }

    // Adjustment for implicit hydrogens (simple approximation)
    let h_count = estimate_implicit_hydrogens(smiles);
    total_mass + h_count as f32 * 1.00784
}

/// Calculate logP (partition coefficient) from SMILES
/// Simplified calculation based on functional groups
pub fn logp_from_smiles(smiles: &str) -> f32 {
    let mut logp = 0.0;

    // Hydrophobic contributions
    let c_count = smiles.chars().filter(|&c| c == 'C').count() as f32;
    logp += c_count * 0.5; // Each carbon contributes ~0.5 to logP

    // Hydrophilic contributions
    let o_count = smiles.chars().filter(|&c| c == 'O').count() as f32;
    logp -= o_count * 0.8; // Oxygen decreases logP

    let n_count = smiles.chars().filter(|&c| c == 'N').count() as f32;
    logp -= n_count * 0.5; // Nitrogen decreases logP

    // Halogens increase logP
    let f_count = smiles.chars().filter(|&c| c == 'F').count() as f32;
    logp += f_count * 0.3;

    // Special bonds
    let double_bonds = smiles.chars().filter(|&c| c == '=').count() as f32;
    logp += double_bonds * 0.1;

    let triple_bonds = smiles.chars().filter(|&c| c == '#').count() as f32;
    logp += triple_bonds * 0.2;

    // Ring systems (indicated by numbers) tend to increase logP
    let rings = smiles.chars().filter(|c| c.is_numeric()).count() as f32 / 2.0;
    logp += rings * 0.3;

    logp.clamp(-2.0, 7.0) // Typical range of logP
}

/// Calculate polar surface area from SMILES
/// PSA approximation based on polar atoms
pub fn polar_surface_area_from_smiles(smiles: &str) -> f32 {
    let mut psa = 0.0;

    // Oxygen in different contexts
    let o_count = smiles.chars().filter(|&c| c == 'O').count() as f32;
    psa += o_count * 20.23; // Average value for oxygen

    // Nitrogen
    let n_count = smiles.chars().filter(|&c| c == 'N').count() as f32;
    psa += n_count * 26.30; // Average value for nitrogen

    // Sulfur contributes less
    let s_count = smiles.chars().filter(|&c| c == 'S').count() as f32;
    psa += s_count * 5.0;

    psa
}

/// Count hydrogen bond donors and acceptors
pub fn hbd_hba_count(smiles: &str) -> (usize, usize) {
    let mut hbd = 0; // Hydrogen bond donors
    let mut hba = 0; // Hydrogen bond acceptors

    // Count polar atoms
    let o_count = smiles.chars().filter(|&c| c == 'O').count();
    let n_count = smiles.chars().filter(|&c| c == 'N').count();

    // Oxygen: 1 acceptor per O, potentially 1 donor (OH groups)
    hba += o_count;
    hbd += o_count / 2; // Rough estimate

    // Nitrogen: 1 acceptor per N, potentially 1-2 donors (NH, NH2)
    hba += n_count;
    hbd += n_count;

    (hbd, hba)
}

/// Count rotatable bonds (simplified)
pub fn rotatable_bonds_count(smiles: &str) -> usize {
    // Simple estimate: single bonds between non-terminal heavy atoms
    let single_bonds = smiles.len().saturating_sub(
        smiles.chars().filter(|&c| c == '=' || c == '#' || c == '(' || c == ')').count()
    );
    
    // Rough estimate
    single_bonds.saturating_sub(5) / 2
}

/// Count heavy atoms (non-hydrogen)
pub fn heavy_atom_count(smiles: &str) -> usize {
    smiles.chars().filter(|c| c.is_uppercase()).count()
}

/// Check Lipinski's Rule of Five compliance
pub fn lipinski_violations(smiles: &str) -> usize {
    let mw = molecular_weight_from_smiles(smiles);
    let logp = logp_from_smiles(smiles);
    let (hbd, hba) = hbd_hba_count(smiles);
    
    let mut violations = 0;
    
    if mw > 500.0 { violations += 1; }
    if logp > 5.0 { violations += 1; }
    if hbd > 5 { violations += 1; }
    if hba > 10 { violations += 1; }
    
    violations
}

fn get_atomic_masses() -> HashMap<String, f32> {
    let mut masses = HashMap::new();
    masses.insert("H".to_string(), 1.00784);
    masses.insert("C".to_string(), 12.011);
    masses.insert("N".to_string(), 14.0067);
    masses.insert("O".to_string(), 15.999);
    masses.insert("S".to_string(), 32.06);
    masses.insert("P".to_string(), 30.9738);
    masses.insert("F".to_string(), 18.9984);
    masses.insert("Cl".to_string(), 35.453);
    masses.insert("Br".to_string(), 79.904);
    masses.insert("I".to_string(), 126.904);
    masses
}

fn estimate_implicit_hydrogens(smiles: &str) -> usize {
    // Simple estimation of implicit hydrogens
    // In a real SMILES parser, this would be more complex
    
    let c_count = smiles.chars().filter(|&c| c == 'C').count();
    let n_count = smiles.chars().filter(|&c| c == 'N').count();
    let o_count = smiles.chars().filter(|&c| c == 'O').count();
    
    // Double/triple bonds reduce hydrogen count
    let double_bonds = smiles.chars().filter(|&c| c == '=').count();
    let triple_bonds = smiles.chars().filter(|&c| c == '#').count();
    
    // Approximation: C has 4 valence, N has 3, O has 2
    // Each bond uses one valence
    let base_h = c_count * 2 + n_count + o_count.saturating_sub(1);
    base_h.saturating_sub(double_bonds + triple_bonds * 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_molecular_weight() {
        // Methane CH4 ≈ 16
        let mw = molecular_weight_from_smiles("C");
        assert!(mw > 10.0 && mw < 20.0);
    }

    #[test]
    fn test_logp() {
        // Hydrophobic molecule should have positive logP
        let logp = logp_from_smiles("CCCCCCCC");
        assert!(logp > 0.0);
    }
}
