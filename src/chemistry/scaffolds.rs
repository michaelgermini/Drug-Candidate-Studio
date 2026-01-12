//! Pharmaceutical scaffolds - Real drug templates
//! Based on known drug molecules and common pharmacophores

use rand::Rng;
use rand::rngs::StdRng;

/// Known drug scaffolds with their properties
#[derive(Clone, Debug)]
pub struct DrugScaffold {
    pub name: &'static str,
    pub smiles: &'static str,
    pub category: &'static str,
    pub mw_approx: f32,
}

/// Database of real pharmaceutical scaffolds
pub const DRUG_SCAFFOLDS: &[DrugScaffold] = &[
    // Analgesics / Anti-inflammatory
    DrugScaffold {
        name: "Aspirin",
        smiles: "CC(=O)Oc1ccccc1C(=O)O",
        category: "NSAID",
        mw_approx: 180.0,
    },
    DrugScaffold {
        name: "Ibuprofen",
        smiles: "CC(C)Cc1ccc(cc1)C(C)C(=O)O",
        category: "NSAID",
        mw_approx: 206.0,
    },
    DrugScaffold {
        name: "Paracetamol",
        smiles: "CC(=O)Nc1ccc(O)cc1",
        category: "Analgesic",
        mw_approx: 151.0,
    },
    DrugScaffold {
        name: "Naproxen",
        smiles: "COc1ccc2cc(ccc2c1)C(C)C(=O)O",
        category: "NSAID",
        mw_approx: 230.0,
    },
    
    // Antibiotics
    DrugScaffold {
        name: "Penicillin-core",
        smiles: "CC1(C)SC2C(NC(=O)C)C(=O)N2C1C(=O)O",
        category: "Antibiotic",
        mw_approx: 334.0,
    },
    DrugScaffold {
        name: "Sulfanilamide",
        smiles: "Nc1ccc(cc1)S(N)(=O)=O",
        category: "Antibiotic",
        mw_approx: 172.0,
    },
    DrugScaffold {
        name: "Ciprofloxacin-core",
        smiles: "c1cc2c(cc1F)c(=O)c(cn2C3CC3)C(=O)O",
        category: "Antibiotic",
        mw_approx: 331.0,
    },
    
    // Cardiovascular
    DrugScaffold {
        name: "Atenolol",
        smiles: "CC(C)NCC(O)COc1ccc(cc1)CC(N)=O",
        category: "Beta-blocker",
        mw_approx: 266.0,
    },
    DrugScaffold {
        name: "Propranolol",
        smiles: "CC(C)NCC(O)COc1cccc2ccccc12",
        category: "Beta-blocker",
        mw_approx: 259.0,
    },
    DrugScaffold {
        name: "Lisinopril-core",
        smiles: "NCCCC(N)C(=O)N1CCCC1C(=O)O",
        category: "ACE-inhibitor",
        mw_approx: 405.0,
    },
    
    // CNS drugs
    DrugScaffold {
        name: "Diazepam-core",
        smiles: "CN1C(=O)CN=C(c2ccccc2)c3cc(Cl)ccc13",
        category: "Benzodiazepine",
        mw_approx: 284.0,
    },
    DrugScaffold {
        name: "Fluoxetine",
        smiles: "CNCCC(Oc1ccc(cc1)C(F)(F)F)c2ccccc2",
        category: "SSRI",
        mw_approx: 309.0,
    },
    DrugScaffold {
        name: "Sertraline-core",
        smiles: "CNC1CCC(c2ccc(Cl)c(Cl)c2)c3ccccc13",
        category: "SSRI",
        mw_approx: 306.0,
    },
    DrugScaffold {
        name: "Caffeine",
        smiles: "Cn1cnc2c1c(=O)n(c(=O)n2C)C",
        category: "Stimulant",
        mw_approx: 194.0,
    },
    
    // Antihistamines
    DrugScaffold {
        name: "Diphenhydramine",
        smiles: "CN(C)CCOC(c1ccccc1)c2ccccc2",
        category: "Antihistamine",
        mw_approx: 255.0,
    },
    DrugScaffold {
        name: "Loratadine-core",
        smiles: "CCOC(=O)N1CCC(=C2c3ccc(Cl)cc3CCc4cccnc24)CC1",
        category: "Antihistamine",
        mw_approx: 382.0,
    },
    
    // Antidiabetics
    DrugScaffold {
        name: "Metformin",
        smiles: "CN(C)C(=N)NC(=N)N",
        category: "Antidiabetic",
        mw_approx: 129.0,
    },
    DrugScaffold {
        name: "Glipizide-core",
        smiles: "Cc1cnc(cn1)C(=O)NCCc2ccc(cc2)S(=O)(=O)NC(=O)N",
        category: "Antidiabetic",
        mw_approx: 445.0,
    },
    
    // Antiviral
    DrugScaffold {
        name: "Acyclovir",
        smiles: "Nc1nc2c(ncn2COCCO)c(=O)[nH]1",
        category: "Antiviral",
        mw_approx: 225.0,
    },
    DrugScaffold {
        name: "Oseltamivir-core",
        smiles: "CCOC(=O)C1=CC(OC(CC)CC)C(NC(C)=O)C(N)C1",
        category: "Antiviral",
        mw_approx: 312.0,
    },
    
    // Anticancer
    DrugScaffold {
        name: "Imatinib-core",
        smiles: "Cc1ccc(NC(=O)c2ccc(CN3CCN(C)CC3)cc2)cc1Nc4nccc(n4)c5cccnc5",
        category: "Kinase-inhibitor",
        mw_approx: 493.0,
    },
    DrugScaffold {
        name: "Methotrexate-core",
        smiles: "CN(Cc1cnc2nc(N)nc(N)c2n1)c3ccc(cc3)C(=O)NC(CCC(=O)O)C(=O)O",
        category: "Antimetabolite",
        mw_approx: 454.0,
    },
    
    // Common heterocyclic scaffolds
    DrugScaffold {
        name: "Benzimidazole",
        smiles: "c1ccc2[nH]cnc2c1",
        category: "Scaffold",
        mw_approx: 118.0,
    },
    DrugScaffold {
        name: "Quinoline",
        smiles: "c1ccc2ncccc2c1",
        category: "Scaffold",
        mw_approx: 129.0,
    },
    DrugScaffold {
        name: "Indole",
        smiles: "c1ccc2[nH]ccc2c1",
        category: "Scaffold",
        mw_approx: 117.0,
    },
    DrugScaffold {
        name: "Pyrimidine",
        smiles: "c1cncnc1",
        category: "Scaffold",
        mw_approx: 80.0,
    },
    DrugScaffold {
        name: "Piperidine",
        smiles: "C1CCNCC1",
        category: "Scaffold",
        mw_approx: 85.0,
    },
    DrugScaffold {
        name: "Morpholine",
        smiles: "C1COCCN1",
        category: "Scaffold",
        mw_approx: 87.0,
    },
    DrugScaffold {
        name: "Piperazine",
        smiles: "C1CNCCN1",
        category: "Scaffold",
        mw_approx: 86.0,
    },
    DrugScaffold {
        name: "Thiazole",
        smiles: "c1cscn1",
        category: "Scaffold",
        mw_approx: 85.0,
    },
    DrugScaffold {
        name: "Oxazole",
        smiles: "c1cocn1",
        category: "Scaffold",
        mw_approx: 69.0,
    },
    DrugScaffold {
        name: "Triazole",
        smiles: "c1cn[nH]n1",
        category: "Scaffold",
        mw_approx: 69.0,
    },
];

/// Common substituents for scaffold decoration
pub const SUBSTITUENTS: &[(&str, &str)] = &[
    ("methyl", "C"),
    ("ethyl", "CC"),
    ("propyl", "CCC"),
    ("isopropyl", "C(C)C"),
    ("tert-butyl", "C(C)(C)C"),
    ("hydroxyl", "O"),
    ("methoxy", "OC"),
    ("ethoxy", "OCC"),
    ("amino", "N"),
    ("dimethylamino", "N(C)C"),
    ("fluoro", "F"),
    ("chloro", "Cl"),
    ("bromo", "Br"),
    ("cyano", "C#N"),
    ("nitro", "N(=O)=O"),
    ("carboxyl", "C(=O)O"),
    ("amide", "C(=O)N"),
    ("sulfonyl", "S(=O)(=O)"),
    ("acetyl", "C(=O)C"),
    ("phenyl", "c1ccccc1"),
    ("benzyl", "Cc1ccccc1"),
    ("pyridyl", "c1ccncc1"),
];

/// Generate a SMILES based on a real drug scaffold with modifications
pub fn generate_from_scaffold(rng: &mut StdRng) -> String {
    let scaffold = &DRUG_SCAFFOLDS[rng.gen_range(0..DRUG_SCAFFOLDS.len())];
    let mut smiles = scaffold.smiles.to_string();
    
    // Optionally add substituents
    let num_subs = rng.gen_range(0..=2);
    for _ in 0..num_subs {
        let (_, sub_smiles) = SUBSTITUENTS[rng.gen_range(0..SUBSTITUENTS.len())];
        // Add substituent to end (simplified attachment)
        if rng.gen_bool(0.5) {
            smiles.push_str(sub_smiles);
        }
    }
    
    smiles
}

/// Generate a novel scaffold by combining fragments
pub fn generate_hybrid_scaffold(rng: &mut StdRng) -> String {
    // Pick two scaffolds and combine concepts
    let scaffold1 = &DRUG_SCAFFOLDS[rng.gen_range(0..DRUG_SCAFFOLDS.len())];
    let scaffold2 = &DRUG_SCAFFOLDS[rng.gen_range(0..DRUG_SCAFFOLDS.len())];
    
    // Use one as base, add substituent from another category
    let mut smiles = scaffold1.smiles.to_string();
    
    // Add a linker and fragment
    let linkers = ["", "C", "CC", "O", "N", "C(=O)N"];
    let linker = linkers[rng.gen_range(0..linkers.len())];
    
    if rng.gen_bool(0.3) && scaffold2.mw_approx < 200.0 {
        smiles.push_str(linker);
        // Add small scaffold fragment
        if scaffold2.smiles.len() < 20 {
            smiles.push_str(scaffold2.smiles);
        }
    }
    
    smiles
}

/// Get scaffold information by name
pub fn get_scaffold_by_name(name: &str) -> Option<&'static DrugScaffold> {
    DRUG_SCAFFOLDS.iter().find(|s| s.name.eq_ignore_ascii_case(name))
}

/// Get all scaffolds in a category
pub fn get_scaffolds_by_category(category: &str) -> Vec<&'static DrugScaffold> {
    DRUG_SCAFFOLDS
        .iter()
        .filter(|s| s.category.eq_ignore_ascii_case(category))
        .collect()
}

/// List all available categories
pub fn list_categories() -> Vec<&'static str> {
    let mut categories: Vec<_> = DRUG_SCAFFOLDS.iter().map(|s| s.category).collect();
    categories.sort();
    categories.dedup();
    categories
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_scaffold_count() {
        assert!(DRUG_SCAFFOLDS.len() >= 30);
    }

    #[test]
    fn test_generate_from_scaffold() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..20 {
            let smiles = generate_from_scaffold(&mut rng);
            assert!(!smiles.is_empty());
        }
    }

    #[test]
    fn test_categories() {
        let categories = list_categories();
        assert!(categories.contains(&"NSAID"));
        assert!(categories.contains(&"Antibiotic"));
    }
}
