//! Drug-likeness rules: Lipinski, Veber, and toxicity alerts (PAINS)

use super::descriptors;

/// Lipinski's Rule of Five results
#[derive(Clone, Debug, Default)]
pub struct LipinskiResult {
    pub mw_ok: bool,           // MW <= 500
    pub logp_ok: bool,         // LogP <= 5
    pub hbd_ok: bool,          // HBD <= 5
    pub hba_ok: bool,          // HBA <= 10
    pub violations: u8,
    pub passed: bool,          // <= 1 violation
}

/// Veber rules for oral bioavailability
#[derive(Clone, Debug, Default)]
pub struct VeberResult {
    pub rotatable_bonds_ok: bool,  // <= 10
    pub psa_ok: bool,              // <= 140 Å²
    pub passed: bool,
}

/// PAINS (Pan-Assay Interference Compounds) alert
#[derive(Clone, Debug)]
pub struct PainsAlert {
    pub name: &'static str,
    pub pattern: &'static str,
    pub severity: &'static str,  // "high", "medium", "low"
}

/// Combined drug-likeness assessment
#[derive(Clone, Debug, Default)]
pub struct DrugLikenessResult {
    pub lipinski: LipinskiResult,
    pub veber: VeberResult,
    pub pains_alerts: Vec<String>,
    pub overall_score: f32,  // 0-1, higher is better
    pub recommendation: String,
}

/// Check Lipinski's Rule of Five
pub fn check_lipinski(smiles: &str) -> LipinskiResult {
    let mw = descriptors::molecular_weight_from_smiles(smiles);
    let logp = descriptors::logp_from_smiles(smiles);
    let (hbd, hba) = descriptors::hbd_hba_count(smiles);
    
    let mw_ok = mw <= 500.0;
    let logp_ok = logp <= 5.0;
    let hbd_ok = hbd <= 5;
    let hba_ok = hba <= 10;
    
    let violations = (!mw_ok as u8) + (!logp_ok as u8) + (!hbd_ok as u8) + (!hba_ok as u8);
    
    LipinskiResult {
        mw_ok,
        logp_ok,
        hbd_ok,
        hba_ok,
        violations,
        passed: violations <= 1,
    }
}

/// Check Veber rules for oral bioavailability
pub fn check_veber(smiles: &str) -> VeberResult {
    let rotatable_bonds = count_rotatable_bonds(smiles);
    let psa = descriptors::polar_surface_area_from_smiles(smiles);
    
    let rotatable_bonds_ok = rotatable_bonds <= 10;
    let psa_ok = psa <= 140.0;
    
    VeberResult {
        rotatable_bonds_ok,
        psa_ok,
        passed: rotatable_bonds_ok && psa_ok,
    }
}

/// Count rotatable bonds (simplified)
pub fn count_rotatable_bonds(smiles: &str) -> usize {
    // Count single bonds between non-terminal, non-ring heavy atoms
    // Simplified: count single bonds minus ring bonds and terminal bonds
    
    let total_atoms = smiles.chars().filter(|c| c.is_alphabetic() && c.is_uppercase()).count();
    let ring_indicators = smiles.chars().filter(|c| c.is_numeric()).count() / 2;
    let double_bonds = smiles.chars().filter(|&c| c == '=').count();
    let triple_bonds = smiles.chars().filter(|&c| c == '#').count();
    let branches = smiles.chars().filter(|&c| c == '(').count();
    
    // Estimate: total bonds - ring bonds - multiple bonds - terminal bonds
    let total_bonds = total_atoms.saturating_sub(1) + ring_indicators;
    let fixed_bonds = ring_indicators + double_bonds + triple_bonds;
    let terminal_estimate = smiles.chars()
        .filter(|&c| c == 'F' || c == 'I')
        .count() + smiles.matches("Cl").count() + smiles.matches("Br").count();
    
    total_bonds.saturating_sub(fixed_bonds).saturating_sub(terminal_estimate).saturating_sub(branches)
}

/// PAINS patterns - substructures that cause assay interference
const PAINS_PATTERNS: &[PainsAlert] = &[
    // Reactive/Toxic groups
    PainsAlert { name: "Aldehyde", pattern: "C=O", severity: "medium" },
    PainsAlert { name: "Michael acceptor", pattern: "C=CC=O", severity: "high" },
    PainsAlert { name: "Epoxide", pattern: "C1OC1", severity: "high" },
    PainsAlert { name: "Aziridine", pattern: "C1NC1", severity: "high" },
    PainsAlert { name: "Acyl halide", pattern: "C(=O)Cl", severity: "high" },
    PainsAlert { name: "Sulfonyl halide", pattern: "S(=O)(=O)Cl", severity: "high" },
    PainsAlert { name: "Isocyanate", pattern: "N=C=O", severity: "high" },
    PainsAlert { name: "Isothiocyanate", pattern: "N=C=S", severity: "high" },
    
    // Frequent hitters
    PainsAlert { name: "Quinone", pattern: "C1=CC(=O)C=CC1=O", severity: "high" },
    PainsAlert { name: "Rhodanine", pattern: "S=C1NC(=O)CS1", severity: "high" },
    PainsAlert { name: "Catechol", pattern: "c1ccc(O)c(O)c1", severity: "medium" },
    PainsAlert { name: "Resorcinol", pattern: "c1cc(O)cc(O)c1", severity: "medium" },
    PainsAlert { name: "Phenol-ester", pattern: "c1ccccc1OC(=O)", severity: "medium" },
    
    // Unstable groups
    PainsAlert { name: "Hydrazine", pattern: "NN", severity: "medium" },
    PainsAlert { name: "Hydroxylamine", pattern: "NO", severity: "medium" },
    PainsAlert { name: "Peroxide", pattern: "OO", severity: "high" },
    PainsAlert { name: "Disulfide", pattern: "SS", severity: "medium" },
    PainsAlert { name: "Thiol", pattern: "SH", severity: "low" },
    
    // Genotoxic alerts
    PainsAlert { name: "Nitro-aromatic", pattern: "c1ccccc1N(=O)=O", severity: "high" },
    PainsAlert { name: "Azide", pattern: "N=[N+]=[N-]", severity: "high" },
    PainsAlert { name: "Nitroso", pattern: "N=O", severity: "high" },
    
    // Metabolic liabilities
    PainsAlert { name: "Aniline", pattern: "c1ccccc1N", severity: "low" },
    PainsAlert { name: "Thiourea", pattern: "NC(=S)N", severity: "medium" },
];

/// Check for PAINS alerts
pub fn check_pains(smiles: &str) -> Vec<String> {
    let mut alerts = Vec::new();
    let smiles_lower = smiles.to_lowercase();
    
    for alert in PAINS_PATTERNS {
        // Simple substring matching (real implementation would use SMARTS)
        if contains_substructure(smiles, alert.pattern) {
            alerts.push(format!("{} ({})", alert.name, alert.severity));
        }
    }
    
    // Additional specific checks
    if smiles_lower.contains("nn") && !smiles_lower.contains("nnn") {
        if !alerts.iter().any(|a| a.contains("Hydrazine")) {
            alerts.push("Hydrazine-like (medium)".to_string());
        }
    }
    
    // Check for too many halogens
    let halogen_count = smiles.matches('F').count() 
        + smiles.matches("Cl").count() 
        + smiles.matches("Br").count()
        + smiles.matches('I').count();
    if halogen_count > 4 {
        alerts.push("Excessive halogens (medium)".to_string());
    }
    
    alerts
}

/// Simple substructure check (pattern matching)
fn contains_substructure(smiles: &str, pattern: &str) -> bool {
    // Simplified check - real implementation would use SMARTS matching
    let smiles_normalized = smiles.replace("(", "").replace(")", "");
    let pattern_normalized = pattern.replace("(", "").replace(")", "");
    
    smiles_normalized.contains(&pattern_normalized)
}

/// Comprehensive drug-likeness assessment
pub fn assess_druglikeness(smiles: &str) -> DrugLikenessResult {
    let lipinski = check_lipinski(smiles);
    let veber = check_veber(smiles);
    let pains_alerts = check_pains(smiles);
    
    // Calculate overall score
    let mut score = 1.0f32;
    
    // Lipinski penalties
    score -= lipinski.violations as f32 * 0.15;
    
    // Veber penalties
    if !veber.rotatable_bonds_ok { score -= 0.1; }
    if !veber.psa_ok { score -= 0.1; }
    
    // PAINS penalties
    for alert in &pains_alerts {
        if alert.contains("high") {
            score -= 0.2;
        } else if alert.contains("medium") {
            score -= 0.1;
        } else {
            score -= 0.05;
        }
    }
    
    let overall_score = score.clamp(0.0, 1.0);
    
    // Generate recommendation
    let recommendation = if overall_score >= 0.8 && pains_alerts.is_empty() {
        "Excellent drug-like properties".to_string()
    } else if overall_score >= 0.6 && pains_alerts.len() <= 1 {
        "Good candidate, minor concerns".to_string()
    } else if overall_score >= 0.4 {
        "Moderate - optimization recommended".to_string()
    } else {
        "Poor drug-likeness - significant issues".to_string()
    };
    
    DrugLikenessResult {
        lipinski,
        veber,
        pains_alerts,
        overall_score,
        recommendation,
    }
}

/// Quick drug-likeness score (0-1)
pub fn quick_druglikeness_score(smiles: &str) -> f32 {
    assess_druglikeness(smiles).overall_score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lipinski_aspirin() {
        // Aspirin should pass Lipinski
        let result = check_lipinski("CC(=O)Oc1ccccc1C(=O)O");
        assert!(result.passed);
        assert_eq!(result.violations, 0);
    }

    #[test]
    fn test_veber() {
        let result = check_veber("CCCC");
        assert!(result.passed);
    }

    #[test]
    fn test_pains_detection() {
        // Epoxide should trigger alert
        let alerts = check_pains("C1OC1CC");
        assert!(!alerts.is_empty());
    }

    #[test]
    fn test_overall_assessment() {
        let result = assess_druglikeness("c1ccccc1");  // Benzene
        assert!(result.overall_score > 0.5);
    }
}
