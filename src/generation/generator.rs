use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rayon::prelude::*;

use crate::app::state::Candidate;
use crate::chemistry;

/// Generate a batch of drug candidates with valid SMILES and computed properties
pub fn generate_candidates(start_id: usize, n: usize, seed: u64) -> Vec<Candidate> {
    let mut rng = StdRng::seed_from_u64(seed);

    (0..n).map(|i| {
        let id = start_id + i;
        
        // Mix scaffold-based and random generation
        let smiles = if rng.gen_bool(0.6) {
            // Use pharmaceutical scaffolds 60% of the time
            chemistry::scaffolds::generate_from_scaffold(&mut rng)
        } else if rng.gen_bool(0.3) {
            // Use hybrid scaffolds 12% of the time
            chemistry::scaffolds::generate_hybrid_scaffold(&mut rng)
        } else {
            // Random generation 28% of the time
            chemistry::smiles::generate_safe_smiles(&mut rng)
        };

        let properties = calculate_properties(&smiles, &mut rng);

        Candidate {
            id,
            smiles,
            efficacy: properties.efficacy,
            toxicity: properties.toxicity,
            synthesis_cost: properties.synthesis_cost,
            manufacturing_cost: properties.manufacturing_cost,
            pareto: false,
        }
    }).collect()
}

/// Generate candidates in parallel using all CPU cores
pub fn generate_candidates_parallel(start_id: usize, n: usize, seed: u64) -> Vec<Candidate> {
    let candidates: Vec<Candidate> = (0..n)
        .into_par_iter()
        .map(|i| {
            let thread_seed = seed.wrapping_add(i as u64 * 31337);
            let mut rng = StdRng::seed_from_u64(thread_seed);
            
            let id = start_id + i;
            
            // Mix scaffold-based and random generation
            let smiles = if rng.gen_bool(0.6) {
                chemistry::scaffolds::generate_from_scaffold(&mut rng)
            } else if rng.gen_bool(0.3) {
                chemistry::scaffolds::generate_hybrid_scaffold(&mut rng)
            } else {
                chemistry::smiles::generate_safe_smiles(&mut rng)
            };
            
            let properties = calculate_properties(&smiles, &mut rng);

            Candidate {
                id,
                smiles,
                efficacy: properties.efficacy,
                toxicity: properties.toxicity,
                synthesis_cost: properties.synthesis_cost,
                manufacturing_cost: properties.manufacturing_cost,
                pareto: false,
            }
        })
        .collect();

    candidates
}

#[derive(Clone)]
struct MolecularProperties {
    efficacy: f32,
    toxicity: f32,
    synthesis_cost: f32,
    manufacturing_cost: f32,
}

fn calculate_properties(smiles: &str, rng: &mut StdRng) -> MolecularProperties {
    // Use real chemical properties
    let mw = chemistry::descriptors::molecular_weight_from_smiles(smiles);
    let logp = chemistry::descriptors::logp_from_smiles(smiles);
    let psa = chemistry::descriptors::polar_surface_area_from_smiles(smiles);
    let (hbd, hba) = chemistry::descriptors::hbd_hba_count(smiles);

    // Calculate objectives from real properties
    let efficacy = calculate_efficacy_from_properties(mw, logp, psa, hbd, hba, rng);
    let toxicity = calculate_toxicity_from_properties(mw, logp, psa, hbd, hba, rng);
    let synthesis_cost = calculate_synthesis_cost_from_properties(smiles, mw);
    let manufacturing_cost = calculate_manufacturing_cost_from_properties(mw, logp);

    MolecularProperties {
        efficacy: efficacy.clamp(0.0, 1.0),
        toxicity: toxicity.clamp(0.0, 1.0),
        synthesis_cost: synthesis_cost.clamp(0.0, 1.0),
        manufacturing_cost: manufacturing_cost.clamp(0.0, 1.0),
    }
}

fn calculate_efficacy_from_properties(
    mw: f32, 
    logp: f32, 
    psa: f32, 
    hbd: usize, 
    hba: usize,
    rng: &mut StdRng
) -> f32 {
    // Efficacy based on Lipinski's Rule of Five
    let mut score = 0.5;

    // Bonus for MW in optimal range (200-500)
    if mw >= 200.0 && mw <= 500.0 {
        score += 0.2;
    } else if mw > 500.0 {
        score -= 0.15;
    } else if mw < 150.0 {
        score -= 0.1;
    }

    // Bonus for logP in optimal range (1-4)
    if logp >= 1.0 && logp <= 4.0 {
        score += 0.2;
    } else if logp < 0.0 || logp > 5.0 {
        score -= 0.1;
    }

    // Bonus for PSA in oral bioavailability range (20-140)
    if psa >= 20.0 && psa <= 140.0 {
        score += 0.15;
    }

    // Bonus for H-bond donors/acceptors within limits
    if hbd <= 5 && hba <= 10 {
        score += 0.1;
    }

    // Add some random variation (biological variability)
    score += rng.gen_range(-0.1..0.1);

    score
}

fn calculate_toxicity_from_properties(
    mw: f32, 
    logp: f32, 
    psa: f32, 
    hbd: usize, 
    hba: usize,
    rng: &mut StdRng
) -> f32 {
    let mut toxicity = 0.1;

    // Higher logP (hydrophobic) molecules tend to be more toxic
    if logp > 5.0 {
        toxicity += 0.25;
    } else if logp > 4.0 {
        toxicity += 0.1;
    }

    // Very large molecules can be problematic
    if mw > 600.0 {
        toxicity += 0.2;
    } else if mw > 500.0 {
        toxicity += 0.1;
    }

    // Too many H-bond sites can indicate reactivity
    if hbd > 5 || hba > 10 {
        toxicity += 0.1;
    }

    // Very low PSA can indicate membrane disruption
    if psa < 20.0 {
        toxicity += 0.15;
    }

    // Random biological variation
    toxicity += rng.gen_range(-0.05..0.15);

    toxicity
}

fn calculate_synthesis_cost_from_properties(smiles: &str, mw: f32) -> f32 {
    let mut cost = 0.1;

    // Structural complexity
    let ring_count = smiles.chars().filter(|c| c.is_numeric()).count() as f32 / 2.0;
    cost += ring_count * 0.08;

    let double_bonds = smiles.chars().filter(|&c| c == '=').count() as f32;
    cost += double_bonds * 0.04;

    let triple_bonds = smiles.chars().filter(|&c| c == '#').count() as f32;
    cost += triple_bonds * 0.08;

    let branches = smiles.chars().filter(|&c| c == '(').count() as f32;
    cost += branches * 0.05;

    // Exotic atoms are more expensive
    let halogens = smiles.chars().filter(|&c| "FClBr".contains(c)).count() as f32;
    cost += halogens * 0.03;

    // Aromatic rings add complexity
    let aromatic = smiles.chars().filter(|c| c.is_lowercase() && c.is_alphabetic()).count() as f32;
    cost += aromatic * 0.02;

    // Size factor
    cost += (mw / 600.0).min(1.0) * 0.2;

    cost
}

fn calculate_manufacturing_cost_from_properties(mw: f32, logp: f32) -> f32 {
    let mut cost = 0.15;

    // Purification cost higher for hydrophobic compounds
    if logp > 4.0 {
        cost += 0.15;
    } else if logp > 3.0 {
        cost += 0.08;
    }

    // Handling cost higher for large compounds
    cost += (mw / 500.0).min(1.0) * 0.25;

    // Very hydrophilic compounds may have stability issues
    if logp < 1.0 {
        cost += 0.1;
    }

    cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_candidates() {
        let candidates = generate_candidates(0, 10, 42);
        assert_eq!(candidates.len(), 10);
        
        for c in &candidates {
            assert!(!c.smiles.is_empty());
            assert!(c.efficacy >= 0.0 && c.efficacy <= 1.0);
            assert!(c.toxicity >= 0.0 && c.toxicity <= 1.0);
            // Verify SMILES validity
            assert!(chemistry::smiles::validate_smiles(&c.smiles), "Invalid: {}", c.smiles);
        }
    }

    #[test]
    fn test_parallel_generation() {
        let candidates = generate_candidates_parallel(0, 100, 42);
        assert_eq!(candidates.len(), 100);
        
        // Check all IDs are unique
        let mut ids: Vec<_> = candidates.iter().map(|c| c.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 100);
    }

    #[test]
    fn test_smiles_variety() {
        let candidates = generate_candidates(0, 100, 42);
        let mut unique_smiles = std::collections::HashSet::new();
        
        for c in &candidates {
            unique_smiles.insert(c.smiles.clone());
        }
        
        // Should have good variety
        assert!(unique_smiles.len() > 50);
    }
}
