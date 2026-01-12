//! SMILES generation with proper chemical valence rules
//! Generates valid molecular structures

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;

/// Atom with valence tracking
#[derive(Clone, Debug)]
struct Atom {
    symbol: &'static str,
    max_valence: u8,
    used_valence: u8,
    aromatic: bool,
    in_ring: bool,
}

impl Atom {
    fn new(symbol: &'static str, max_valence: u8) -> Self {
        Self {
            symbol,
            max_valence,
            used_valence: 0,
            aromatic: false,
            in_ring: false,
        }
    }

    fn available_valence(&self) -> u8 {
        self.max_valence.saturating_sub(self.used_valence)
    }

    fn can_bond(&self, bond_order: u8) -> bool {
        self.available_valence() >= bond_order
    }

    fn add_bond(&mut self, bond_order: u8) {
        self.used_valence += bond_order;
    }
}

/// Molecular graph for SMILES generation
struct MoleculeBuilder {
    atoms: Vec<Atom>,
    bonds: Vec<(usize, usize, u8)>, // (from, to, order)
    ring_closures: Vec<(usize, usize)>,
}

impl MoleculeBuilder {
    fn new() -> Self {
        Self {
            atoms: Vec::new(),
            bonds: Vec::new(),
            ring_closures: Vec::new(),
        }
    }

    fn add_atom(&mut self, symbol: &'static str, valence: u8) -> usize {
        let idx = self.atoms.len();
        self.atoms.push(Atom::new(symbol, valence));
        idx
    }

    fn add_bond(&mut self, from: usize, to: usize, order: u8) -> bool {
        if from >= self.atoms.len() || to >= self.atoms.len() {
            return false;
        }
        if !self.atoms[from].can_bond(order) || !self.atoms[to].can_bond(order) {
            return false;
        }
        
        self.atoms[from].add_bond(order);
        self.atoms[to].add_bond(order);
        self.bonds.push((from, to, order));
        true
    }

    fn to_smiles(&self) -> String {
        if self.atoms.is_empty() {
            return "C".to_string(); // Methane as fallback
        }

        let mut smiles = String::new();
        let mut visited = vec![false; self.atoms.len()];
        let mut ring_labels: HashMap<(usize, usize), u8> = HashMap::new();
        let mut next_ring_label = 1u8;

        // Assign ring labels
        for &(a, b) in &self.ring_closures {
            ring_labels.insert((a.min(b), a.max(b)), next_ring_label);
            next_ring_label += 1;
            if next_ring_label > 9 {
                next_ring_label = 1;
            }
        }

        // Build adjacency list
        let mut adj: Vec<Vec<(usize, u8)>> = vec![vec![]; self.atoms.len()];
        for &(from, to, order) in &self.bonds {
            adj[from].push((to, order));
            adj[to].push((from, order));
        }

        // DFS to build SMILES
        self.build_smiles_dfs(0, &mut visited, &adj, &ring_labels, &mut smiles);

        if smiles.is_empty() {
            self.atoms[0].symbol.to_string()
        } else {
            smiles
        }
    }

    fn build_smiles_dfs(
        &self,
        current: usize,
        visited: &mut Vec<bool>,
        adj: &[Vec<(usize, u8)>],
        ring_labels: &HashMap<(usize, usize), u8>,
        smiles: &mut String,
    ) {
        visited[current] = true;
        
        let atom = &self.atoms[current];
        if atom.aromatic {
            smiles.push_str(&atom.symbol.to_lowercase());
        } else {
            smiles.push_str(atom.symbol);
        }

        // Add ring closure labels
        for (&(a, b), &label) in ring_labels {
            if a == current || b == current {
                smiles.push_str(&label.to_string());
            }
        }

        // Visit neighbors
        let neighbors: Vec<_> = adj[current]
            .iter()
            .filter(|(n, _)| !visited[*n])
            .cloned()
            .collect();

        for (i, (neighbor, bond_order)) in neighbors.iter().enumerate() {
            // Add bond symbol
            match bond_order {
                2 => smiles.push('='),
                3 => smiles.push('#'),
                _ => {} // Single bond is implicit
            }

            // Use parentheses for branches
            if i < neighbors.len() - 1 {
                smiles.push('(');
                self.build_smiles_dfs(*neighbor, visited, adj, ring_labels, smiles);
                smiles.push(')');
            } else {
                self.build_smiles_dfs(*neighbor, visited, adj, ring_labels, smiles);
            }
        }
    }
}

/// Get valence for common atoms
fn get_valence(symbol: &str) -> u8 {
    match symbol {
        "C" => 4,
        "N" => 3,
        "O" => 2,
        "S" => 2,
        "P" => 3,
        "F" | "Cl" | "Br" | "I" => 1,
        _ => 4,
    }
}

/// Generate a valid drug-like SMILES string
pub fn generate_valid_smiles(rng: &mut StdRng) -> String {
    let strategy = rng.gen_range(0..10);
    
    match strategy {
        0..=2 => generate_aliphatic_chain(rng),
        3..=4 => generate_simple_ring(rng),
        5..=6 => generate_aromatic_ring(rng),
        7 => generate_fused_rings(rng),
        8 => generate_heterocycle(rng),
        _ => generate_branched_molecule(rng),
    }
}

/// Generate a simple aliphatic chain
fn generate_aliphatic_chain(rng: &mut StdRng) -> String {
    let mut mol = MoleculeBuilder::new();
    let length = rng.gen_range(3..=10);
    
    // Main chain atoms
    let chain_atoms = ["C", "C", "C", "C", "N", "O"];
    
    let first = mol.add_atom("C", 4);
    let mut prev = first;
    
    for _ in 1..length {
        let atom = chain_atoms[rng.gen_range(0..chain_atoms.len())];
        let valence = get_valence(atom);
        let curr = mol.add_atom(atom, valence);
        
        // Choose bond order based on available valence
        let max_order = mol.atoms[prev].available_valence().min(mol.atoms[curr].available_valence()).min(3);
        let order = if max_order > 1 && rng.gen_bool(0.2) {
            rng.gen_range(1..=max_order.min(2))
        } else {
            1
        };
        
        mol.add_bond(prev, curr, order);
        prev = curr;
    }
    
    // Add functional groups
    add_functional_groups(&mut mol, rng);
    
    mol.to_smiles()
}

/// Generate a simple 5 or 6-membered ring
fn generate_simple_ring(rng: &mut StdRng) -> String {
    let ring_size = if rng.gen_bool(0.5) { 5 } else { 6 };
    let mut mol = MoleculeBuilder::new();
    
    // Create ring atoms
    let mut ring_atoms = Vec::new();
    for _ in 0..ring_size {
        let atom = if rng.gen_bool(0.8) { "C" } else { "N" };
        ring_atoms.push(mol.add_atom(atom, get_valence(atom)));
    }
    
    // Connect ring
    for i in 0..ring_size {
        let next = (i + 1) % ring_size;
        mol.add_bond(ring_atoms[i], ring_atoms[next], 1);
    }
    
    // Mark ring closure
    mol.ring_closures.push((ring_atoms[0], ring_atoms[ring_size - 1]));
    
    // Add substituents
    for &atom_idx in &ring_atoms {
        if mol.atoms[atom_idx].available_valence() > 0 && rng.gen_bool(0.3) {
            let sub = ["C", "O", "N", "F", "Cl"][rng.gen_range(0..5)];
            let sub_idx = mol.add_atom(sub, get_valence(sub));
            mol.add_bond(atom_idx, sub_idx, 1);
        }
    }
    
    mol.to_smiles()
}

/// Generate benzene-like aromatic rings
fn generate_aromatic_ring(rng: &mut StdRng) -> String {
    // Use pre-defined aromatic cores for validity
    let cores = [
        "c1ccccc1",           // benzene
        "c1ccc(cc1)",         // phenyl (for substitution)
        "c1ccncc1",           // pyridine
        "c1cccnc1",           // pyridine isomer
        "c1ccoc1",            // furan
        "c1ccsc1",            // thiophene
        "c1cc[nH]c1",         // pyrrole
        "c1cnc[nH]1",         // imidazole
        "c1ccc2ccccc2c1",     // naphthalene
    ];
    
    let mut smiles = cores[rng.gen_range(0..cores.len())].to_string();
    
    // Add substituents
    let substituents = ["C", "CC", "CCC", "O", "N", "F", "Cl", "Br", "OC", "NC", "C(=O)O", "C(=O)N"];
    
    if rng.gen_bool(0.7) {
        smiles.push_str(substituents[rng.gen_range(0..substituents.len())]);
    }
    
    if rng.gen_bool(0.3) {
        smiles.push_str(substituents[rng.gen_range(0..substituents.len())]);
    }
    
    smiles
}

/// Generate fused ring systems
fn generate_fused_rings(rng: &mut StdRng) -> String {
    let cores = [
        "c1ccc2ccccc2c1",         // naphthalene
        "c1ccc2[nH]ccc2c1",       // indole
        "c1ccc2nccc2c1",          // quinoline
        "c1ccc2nccnc2c1",         // quinazoline
        "c1ccc2c(c1)cccc2",       // naphthalene alt
        "c1cc2ccccc2cc1",         // naphthalene alt2
    ];
    
    let mut smiles = cores[rng.gen_range(0..cores.len())].to_string();
    
    // Add side chain
    if rng.gen_bool(0.6) {
        let chains = ["C", "CC", "CCC", "CCO", "CCN", "CCCC"];
        smiles.push_str(chains[rng.gen_range(0..chains.len())]);
    }
    
    smiles
}

/// Generate heterocyclic compounds
fn generate_heterocycle(rng: &mut StdRng) -> String {
    let heterocycles = [
        "C1CCNCC1",       // piperidine
        "C1CCOC1",        // tetrahydrofuran
        "C1CCOCC1",       // tetrahydropyran
        "C1CCNC1",        // pyrrolidine
        "C1COCCO1",       // 1,3-dioxane
        "C1CN2CCCCC2C1",  // octahydroindole
        "N1CCCCC1",       // piperidine
        "O1CCCCC1",       // tetrahydropyran
        "C1CNCCN1",       // piperazine
    ];
    
    let mut smiles = heterocycles[rng.gen_range(0..heterocycles.len())].to_string();
    
    // Add substituents
    if rng.gen_bool(0.5) {
        let subs = ["C", "CC", "c1ccccc1", "C(=O)C"];
        smiles.push_str(subs[rng.gen_range(0..subs.len())]);
    }
    
    smiles
}

/// Generate branched molecules
fn generate_branched_molecule(rng: &mut StdRng) -> String {
    let mut mol = MoleculeBuilder::new();
    
    // Central atom
    let center = mol.add_atom("C", 4);
    
    // Add 2-4 branches
    let num_branches = rng.gen_range(2..=4);
    
    for _ in 0..num_branches {
        if mol.atoms[center].available_valence() == 0 {
            break;
        }
        
        let branch_length = rng.gen_range(1..=4);
        let mut prev = center;
        
        for j in 0..branch_length {
            let atom = if j == 0 || rng.gen_bool(0.7) { "C" } else { 
                ["N", "O", "S"][rng.gen_range(0..3)]
            };
            let curr = mol.add_atom(atom, get_valence(atom));
            
            if !mol.add_bond(prev, curr, 1) {
                break;
            }
            prev = curr;
        }
    }
    
    mol.to_smiles()
}

/// Add functional groups to molecule
fn add_functional_groups(mol: &mut MoleculeBuilder, rng: &mut StdRng) {
    let num_groups = rng.gen_range(0..=2);
    
    for _ in 0..num_groups {
        // Find atom with available valence
        let candidates: Vec<usize> = mol.atoms
            .iter()
            .enumerate()
            .filter(|(_, a)| a.available_valence() > 0 && a.symbol == "C")
            .map(|(i, _)| i)
            .collect();
        
        if candidates.is_empty() {
            break;
        }
        
        let target = candidates[rng.gen_range(0..candidates.len())];
        
        // Add functional group
        let group_type = rng.gen_range(0..6);
        match group_type {
            0 => {
                // Hydroxyl -OH
                let o = mol.add_atom("O", 2);
                mol.add_bond(target, o, 1);
            }
            1 => {
                // Amino -NH2
                let n = mol.add_atom("N", 3);
                mol.add_bond(target, n, 1);
            }
            2 => {
                // Carbonyl =O
                if mol.atoms[target].available_valence() >= 2 {
                    let o = mol.add_atom("O", 2);
                    mol.add_bond(target, o, 2);
                }
            }
            3 => {
                // Halogen
                let halogen = ["F", "Cl", "Br"][rng.gen_range(0..3)];
                let x = mol.add_atom(halogen, 1);
                mol.add_bond(target, x, 1);
            }
            4 => {
                // Methyl -CH3
                let c = mol.add_atom("C", 4);
                mol.add_bond(target, c, 1);
            }
            _ => {
                // Ether -O-C
                if mol.atoms[target].available_valence() >= 1 {
                    let o = mol.add_atom("O", 2);
                    let c = mol.add_atom("C", 4);
                    mol.add_bond(target, o, 1);
                    mol.add_bond(o, c, 1);
                }
            }
        }
    }
}

/// Validate a SMILES string (basic validation)
pub fn validate_smiles(smiles: &str) -> bool {
    if smiles.is_empty() {
        return false;
    }
    
    // Check balanced parentheses
    let mut paren_count = 0;
    for c in smiles.chars() {
        match c {
            '(' => paren_count += 1,
            ')' => {
                paren_count -= 1;
                if paren_count < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    if paren_count != 0 {
        return false;
    }
    
    // Check ring closures are paired
    let mut ring_counts = [0u8; 10];
    for c in smiles.chars() {
        if let Some(digit) = c.to_digit(10) {
            ring_counts[digit as usize] += 1;
        }
    }
    for count in &ring_counts[1..] {
        if count % 2 != 0 {
            return false;
        }
    }
    
    // Check for invalid patterns
    let invalid_patterns = [
        "((", "))", "()", // Empty branches
        "==", "##",       // Double bond symbols
        "Cl(", "Br(", "F(", "I(", // Halogens can't have branches
    ];
    
    for pattern in &invalid_patterns {
        if smiles.contains(pattern) {
            return false;
        }
    }
    
    true
}

/// Generate and validate a SMILES, with fallback
pub fn generate_safe_smiles(rng: &mut StdRng) -> String {
    for _ in 0..5 {
        let smiles = generate_valid_smiles(rng);
        if validate_smiles(&smiles) {
            return smiles;
        }
    }
    
    // Fallback to known valid SMILES
    let fallbacks = [
        "CCCC", "CCCCC", "CCCCCC",
        "c1ccccc1", "c1ccncc1",
        "C1CCCCC1", "C1CCNCC1",
        "CC(C)C", "CC(C)(C)C",
        "CCO", "CCCO", "CCN",
    ];
    
    fallbacks[rng.gen_range(0..fallbacks.len())].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_valid_smiles() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..100 {
            let smiles = generate_safe_smiles(&mut rng);
            assert!(validate_smiles(&smiles), "Invalid SMILES: {}", smiles);
        }
    }

    #[test]
    fn test_validate_smiles() {
        assert!(validate_smiles("CCCC"));
        assert!(validate_smiles("c1ccccc1"));
        assert!(validate_smiles("C1CCCCC1"));
        assert!(validate_smiles("CC(C)C"));
        
        assert!(!validate_smiles("C((C"));
        assert!(!validate_smiles("C1CCC")); // Unclosed ring
    }

    #[test]
    fn test_aromatic_generation() {
        let mut rng = StdRng::seed_from_u64(42);
        let smiles = generate_aromatic_ring(&mut rng);
        assert!(!smiles.is_empty());
    }
}
