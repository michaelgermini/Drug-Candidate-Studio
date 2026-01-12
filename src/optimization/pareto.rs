use std::collections::HashSet;
use crate::app::state::Candidate;

/// Check if candidate `a` dominates candidate `b` in the multi-objective sense.
/// A dominates B if:
/// - A is at least as good as B in all objectives
/// - A is strictly better than B in at least one objective
/// 
/// Objectives:
/// - efficacy: higher is better
/// - toxicity: lower is better
/// - synthesis_cost: lower is better
/// - manufacturing_cost: lower is better
fn dominates(a: &Candidate, b: &Candidate) -> bool {
    // Check if a is at least as good as b in all objectives
    let at_least_as_good = 
        (a.efficacy >= b.efficacy) &&
        (a.toxicity <= b.toxicity) &&
        (a.synthesis_cost <= b.synthesis_cost) &&
        (a.manufacturing_cost <= b.manufacturing_cost);
    
    // Check if a is strictly better in at least one objective
    let strictly_better = 
        (a.efficacy > b.efficacy) ||
        (a.toxicity < b.toxicity) ||
        (a.synthesis_cost < b.synthesis_cost) ||
        (a.manufacturing_cost < b.manufacturing_cost);
    
    at_least_as_good && strictly_better
}

/// Compute the Pareto front and return the IDs of non-dominated candidates.
/// Uses a simple O(n²) algorithm suitable for moderate dataset sizes.
pub fn pareto_front_ids(cands: &[Candidate]) -> HashSet<usize> {
    let mut front = HashSet::new();

    'outer: for c in cands {
        // Check if any other candidate dominates c
        for other in cands {
            if other.id != c.id && dominates(other, c) {
                // c is dominated, skip it
                continue 'outer;
            }
        }
        // c is not dominated by anyone -> it's on the Pareto front
        front.insert(c.id);
    }

    front
}

/// Compute Pareto front using a more efficient algorithm for larger datasets.
/// Uses non-dominated sorting (NSGA-II style first front extraction).
pub fn pareto_front_ids_fast(cands: &[Candidate]) -> HashSet<usize> {
    if cands.len() < 100 {
        return pareto_front_ids(cands);
    }

    let mut domination_count: Vec<usize> = vec![0; cands.len()];
    
    for i in 0..cands.len() {
        for j in 0..cands.len() {
            if i != j && dominates(&cands[j], &cands[i]) {
                domination_count[i] += 1;
            }
        }
    }

    cands.iter()
        .enumerate()
        .filter(|(i, _)| domination_count[*i] == 0)
        .map(|(_, c)| c.id)
        .collect()
}

/// Calculate crowding distance for diversity preservation
pub fn crowding_distance(cands: &[Candidate], front_ids: &HashSet<usize>) -> Vec<(usize, f32)> {
    let front: Vec<_> = cands.iter().filter(|c| front_ids.contains(&c.id)).collect();
    
    if front.len() <= 2 {
        return front.iter().map(|c| (c.id, f32::INFINITY)).collect();
    }

    let mut distances: std::collections::HashMap<usize, f32> = 
        front.iter().map(|c| (c.id, 0.0)).collect();

    // Calculate distance for each objective
    let objectives: Vec<Box<dyn Fn(&Candidate) -> f32>> = vec![
        Box::new(|c: &Candidate| c.efficacy),
        Box::new(|c: &Candidate| -c.toxicity),
        Box::new(|c: &Candidate| -c.synthesis_cost),
        Box::new(|c: &Candidate| -c.manufacturing_cost),
    ];

    for obj in &objectives {
        let mut sorted: Vec<_> = front.iter().collect();
        sorted.sort_by(|a, b| obj(a).partial_cmp(&obj(b)).unwrap());

        // Boundary points get infinite distance
        if let Some(first) = sorted.first() {
            *distances.get_mut(&first.id).unwrap() = f32::INFINITY;
        }
        if let Some(last) = sorted.last() {
            *distances.get_mut(&last.id).unwrap() = f32::INFINITY;
        }

        // Calculate distance for intermediate points
        let obj_range = obj(sorted.last().unwrap()) - obj(sorted.first().unwrap());
        if obj_range > 0.0 {
            for i in 1..sorted.len() - 1 {
                let dist = (obj(sorted[i + 1]) - obj(sorted[i - 1])) / obj_range;
                *distances.get_mut(&sorted[i].id).unwrap() += dist;
            }
        }
    }

    distances.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(id: usize, eff: f32, tox: f32, syn: f32, mfg: f32) -> Candidate {
        Candidate {
            id,
            smiles: format!("C{}", id),
            efficacy: eff,
            toxicity: tox,
            synthesis_cost: syn,
            manufacturing_cost: mfg,
            pareto: false,
        }
    }

    #[test]
    fn test_dominates() {
        let a = make_candidate(0, 0.8, 0.2, 0.3, 0.3);
        let b = make_candidate(1, 0.6, 0.3, 0.4, 0.4);
        
        assert!(dominates(&a, &b));
        assert!(!dominates(&b, &a));
    }

    #[test]
    fn test_pareto_front() {
        let candidates = vec![
            make_candidate(0, 0.9, 0.1, 0.5, 0.5), // Pareto: high eff, low tox
            make_candidate(1, 0.5, 0.5, 0.1, 0.1), // Pareto: low cost
            make_candidate(2, 0.6, 0.4, 0.4, 0.4), // Dominated
            make_candidate(3, 0.7, 0.3, 0.3, 0.3), // Pareto: balanced
        ];

        let front = pareto_front_ids(&candidates);
        
        assert!(front.contains(&0));
        assert!(front.contains(&1));
        assert!(!front.contains(&2)); // Dominated by candidate 3
        assert!(front.contains(&3));
    }

    #[test]
    fn test_no_domination() {
        // All candidates have trade-offs
        let candidates = vec![
            make_candidate(0, 0.9, 0.9, 0.1, 0.1),
            make_candidate(1, 0.1, 0.1, 0.9, 0.9),
        ];

        let front = pareto_front_ids(&candidates);
        
        assert_eq!(front.len(), 2);
    }
}
