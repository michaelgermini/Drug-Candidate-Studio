//! Molecular similarity calculations using fingerprints
//! Implements Tanimoto coefficient and clustering

use std::collections::{HashMap, HashSet};

/// Molecular fingerprint (bit vector represented as set of "on" bits)
#[derive(Clone, Debug)]
pub struct Fingerprint {
    pub bits: HashSet<u32>,
    pub size: u32,
}

impl Fingerprint {
    pub fn new(size: u32) -> Self {
        Self {
            bits: HashSet::new(),
            size,
        }
    }

    pub fn set_bit(&mut self, bit: u32) {
        if bit < self.size {
            self.bits.insert(bit);
        }
    }

    pub fn count_bits(&self) -> usize {
        self.bits.len()
    }
}

/// Generate a simple path-based fingerprint from SMILES
/// This is a simplified ECFP-like fingerprint
pub fn generate_fingerprint(smiles: &str, size: u32) -> Fingerprint {
    let mut fp = Fingerprint::new(size);
    
    // Hash individual atoms
    for (i, c) in smiles.chars().enumerate() {
        if c.is_alphabetic() {
            let hash = simple_hash(&format!("atom_{}", c)) % size;
            fp.set_bit(hash);
        }
    }
    
    // Hash atom pairs (2-atom paths)
    let chars: Vec<char> = smiles.chars().collect();
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i].is_alphabetic() && chars[i + 1].is_alphabetic() {
            let pair = format!("pair_{}_{}", chars[i], chars[i + 1]);
            let hash = simple_hash(&pair) % size;
            fp.set_bit(hash);
        }
    }
    
    // Hash 3-atom paths
    for i in 0..chars.len().saturating_sub(2) {
        if chars[i].is_alphabetic() && chars[i + 2].is_alphabetic() {
            let triplet = format!("trip_{}_{}_{}", chars[i], chars.get(i + 1).unwrap_or(&'_'), chars[i + 2]);
            let hash = simple_hash(&triplet) % size;
            fp.set_bit(hash);
        }
    }
    
    // Hash ring presence
    let ring_count = smiles.chars().filter(|c| c.is_numeric()).count();
    if ring_count > 0 {
        fp.set_bit(simple_hash("has_ring") % size);
        fp.set_bit(simple_hash(&format!("rings_{}", ring_count / 2)) % size);
    }
    
    // Hash functional groups
    let functional_groups = [
        ("hydroxyl", "O"),
        ("carbonyl", "=O"),
        ("amine", "N"),
        ("halogen_F", "F"),
        ("halogen_Cl", "Cl"),
        ("halogen_Br", "Br"),
        ("aromatic", "c"),
        ("double_bond", "="),
        ("triple_bond", "#"),
        ("ether", "COC"),
        ("ester", "C(=O)O"),
        ("amide", "C(=O)N"),
        ("sulfur", "S"),
        ("phosphorus", "P"),
    ];
    
    for (name, pattern) in functional_groups {
        if smiles.contains(pattern) {
            let hash = simple_hash(&format!("fg_{}", name)) % size;
            fp.set_bit(hash);
        }
    }
    
    // Hash based on size categories
    let atom_count = smiles.chars().filter(|c| c.is_alphabetic() && c.is_uppercase()).count();
    let size_category = atom_count / 5;
    fp.set_bit(simple_hash(&format!("size_{}", size_category)) % size);
    
    fp
}

/// Simple string hash function
fn simple_hash(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for c in s.chars() {
        hash = hash.wrapping_mul(33).wrapping_add(c as u32);
    }
    hash
}

/// Calculate Tanimoto coefficient between two fingerprints
pub fn tanimoto_coefficient(fp1: &Fingerprint, fp2: &Fingerprint) -> f32 {
    let intersection = fp1.bits.intersection(&fp2.bits).count();
    let union = fp1.bits.union(&fp2.bits).count();
    
    if union == 0 {
        return 0.0;
    }
    
    intersection as f32 / union as f32
}

/// Calculate Tanimoto similarity between two SMILES strings
pub fn smiles_similarity(smiles1: &str, smiles2: &str) -> f32 {
    let fp1 = generate_fingerprint(smiles1, 2048);
    let fp2 = generate_fingerprint(smiles2, 2048);
    tanimoto_coefficient(&fp1, &fp2)
}

/// Calculate similarity matrix for a list of SMILES
pub fn similarity_matrix(smiles_list: &[String]) -> Vec<Vec<f32>> {
    let n = smiles_list.len();
    let fingerprints: Vec<Fingerprint> = smiles_list
        .iter()
        .map(|s| generate_fingerprint(s, 2048))
        .collect();
    
    let mut matrix = vec![vec![0.0f32; n]; n];
    
    for i in 0..n {
        matrix[i][i] = 1.0;
        for j in (i + 1)..n {
            let sim = tanimoto_coefficient(&fingerprints[i], &fingerprints[j]);
            matrix[i][j] = sim;
            matrix[j][i] = sim;
        }
    }
    
    matrix
}

/// Simple clustering result
#[derive(Clone, Debug)]
pub struct ClusterResult {
    pub cluster_id: usize,
    pub members: Vec<usize>,  // Indices into original list
    pub centroid_idx: usize,  // Index of most central member
}

/// Cluster molecules using simple leader algorithm
/// threshold: minimum similarity to join cluster (0.0-1.0)
pub fn cluster_molecules(smiles_list: &[String], threshold: f32) -> Vec<ClusterResult> {
    if smiles_list.is_empty() {
        return vec![];
    }
    
    let fingerprints: Vec<Fingerprint> = smiles_list
        .iter()
        .map(|s| generate_fingerprint(s, 2048))
        .collect();
    
    let mut clusters: Vec<ClusterResult> = Vec::new();
    let mut assigned = vec![false; smiles_list.len()];
    
    for i in 0..smiles_list.len() {
        if assigned[i] {
            continue;
        }
        
        // Start new cluster with this molecule as leader
        let mut cluster = ClusterResult {
            cluster_id: clusters.len(),
            members: vec![i],
            centroid_idx: i,
        };
        assigned[i] = true;
        
        // Find similar molecules
        for j in (i + 1)..smiles_list.len() {
            if assigned[j] {
                continue;
            }
            
            let sim = tanimoto_coefficient(&fingerprints[i], &fingerprints[j]);
            if sim >= threshold {
                cluster.members.push(j);
                assigned[j] = true;
            }
        }
        
        // Find centroid (member with highest average similarity to others)
        if cluster.members.len() > 1 {
            let mut best_avg = 0.0f32;
            let mut best_idx = cluster.members[0];
            
            for &m1 in &cluster.members {
                let avg: f32 = cluster.members
                    .iter()
                    .filter(|&&m2| m2 != m1)
                    .map(|&m2| tanimoto_coefficient(&fingerprints[m1], &fingerprints[m2]))
                    .sum::<f32>() / (cluster.members.len() - 1) as f32;
                
                if avg > best_avg {
                    best_avg = avg;
                    best_idx = m1;
                }
            }
            cluster.centroid_idx = best_idx;
        }
        
        clusters.push(cluster);
    }
    
    clusters
}

/// Find the N most similar molecules to a query
pub fn find_similar(query_smiles: &str, database: &[String], top_n: usize) -> Vec<(usize, f32)> {
    let query_fp = generate_fingerprint(query_smiles, 2048);
    
    let mut similarities: Vec<(usize, f32)> = database
        .iter()
        .enumerate()
        .map(|(i, smiles)| {
            let fp = generate_fingerprint(smiles, 2048);
            (i, tanimoto_coefficient(&query_fp, &fp))
        })
        .collect();
    
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    similarities.truncate(top_n);
    similarities
}

/// Calculate diversity of a set of molecules (average pairwise dissimilarity)
pub fn calculate_diversity(smiles_list: &[String]) -> f32 {
    if smiles_list.len() < 2 {
        return 0.0;
    }
    
    let fingerprints: Vec<Fingerprint> = smiles_list
        .iter()
        .map(|s| generate_fingerprint(s, 2048))
        .collect();
    
    let mut total_dissim = 0.0f32;
    let mut count = 0;
    
    for i in 0..fingerprints.len() {
        for j in (i + 1)..fingerprints.len() {
            let sim = tanimoto_coefficient(&fingerprints[i], &fingerprints[j]);
            total_dissim += 1.0 - sim;
            count += 1;
        }
    }
    
    if count > 0 {
        total_dissim / count as f32
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_generation() {
        let fp = generate_fingerprint("CCO", 1024);
        assert!(fp.count_bits() > 0);
    }

    #[test]
    fn test_tanimoto_self() {
        let fp = generate_fingerprint("c1ccccc1", 1024);
        let sim = tanimoto_coefficient(&fp, &fp);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_similar_molecules() {
        // Ethanol and methanol should be somewhat similar
        let sim = smiles_similarity("CCO", "CO");
        assert!(sim > 0.3);
    }

    #[test]
    fn test_dissimilar_molecules() {
        // Benzene and water should be dissimilar
        let sim = smiles_similarity("c1ccccc1", "O");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_clustering() {
        let smiles = vec![
            "CCO".to_string(),
            "CCCO".to_string(),
            "CCCCO".to_string(),
            "c1ccccc1".to_string(),
            "c1ccc(C)cc1".to_string(),
        ];
        
        let clusters = cluster_molecules(&smiles, 0.5);
        assert!(!clusters.is_empty());
    }
}
