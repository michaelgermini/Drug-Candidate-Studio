//! Import/Export functionality: SMILES files, SDF format

use super::state::Candidate;
use std::io::{BufRead, Write};

/// Import SMILES from a text file (one SMILES per line)
pub fn import_smiles_file(path: &str, start_id: usize) -> Result<Vec<Candidate>, String> {
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let reader = std::io::BufReader::new(file);
    let mut candidates = Vec::new();
    let mut id = start_id;
    
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read error: {}", e))?;
        let line = line.trim();
        
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Handle TSV/CSV: take first column as SMILES
        let smiles = line.split(|c| c == '\t' || c == ',' || c == ' ')
            .next()
            .unwrap_or(line)
            .trim();
        
        if !smiles.is_empty() {
            let candidate = create_candidate_from_smiles(id, smiles);
            candidates.push(candidate);
            id += 1;
        }
    }
    
    Ok(candidates)
}

/// Import SMILES from a string (one per line or separated by newlines)
pub fn import_smiles_text(text: &str, start_id: usize) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut id = start_id;
    
    for line in text.lines() {
        let line = line.trim();
        
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        let smiles = line.split(|c| c == '\t' || c == ',' || c == ' ')
            .next()
            .unwrap_or(line)
            .trim();
        
        if !smiles.is_empty() {
            let candidate = create_candidate_from_smiles(id, smiles);
            candidates.push(candidate);
            id += 1;
        }
    }
    
    candidates
}

fn create_candidate_from_smiles(id: usize, smiles: &str) -> Candidate {
    use crate::chemistry::{descriptors, druglikeness};
    
    let mw = descriptors::molecular_weight_from_smiles(smiles);
    let logp = descriptors::logp_from_smiles(smiles);
    let psa = descriptors::polar_surface_area_from_smiles(smiles);
    let (hbd, hba) = descriptors::hbd_hba_count(smiles);
    
    // Calculate properties based on descriptors
    let dl_score = druglikeness::quick_druglikeness_score(smiles);
    
    // Efficacy based on drug-likeness
    let efficacy = dl_score * 0.8 + 0.2 * if mw >= 200.0 && mw <= 500.0 { 1.0 } else { 0.5 };
    
    // Toxicity based on logP and alerts
    let alerts = druglikeness::check_pains(smiles);
    let toxicity = 0.1 + (logp.max(0.0) / 10.0) + (alerts.len() as f32 * 0.1);
    
    // Synthesis cost based on complexity
    let complexity = smiles.len() as f32 / 50.0;
    let synthesis_cost = 0.1 + complexity.min(0.8);
    
    // Manufacturing cost
    let manufacturing_cost = 0.15 + (mw / 1000.0).min(0.5);
    
    Candidate {
        id,
        smiles: smiles.to_string(),
        efficacy: efficacy.clamp(0.0, 1.0),
        toxicity: toxicity.clamp(0.0, 1.0),
        synthesis_cost: synthesis_cost.clamp(0.0, 1.0),
        manufacturing_cost: manufacturing_cost.clamp(0.0, 1.0),
        pareto: false,
    }
}

/// Export candidates to SDF format
pub fn export_sdf(candidates: &[Candidate], path: &str) -> Result<(), String> {
    let mut file = std::fs::File::create(path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    for c in candidates {
        write_sdf_entry(&mut file, c)
            .map_err(|e| format!("Write error: {}", e))?;
    }
    
    Ok(())
}

fn write_sdf_entry<W: Write>(writer: &mut W, candidate: &Candidate) -> std::io::Result<()> {
    // SDF molecule name line
    writeln!(writer, "Candidate_{}", candidate.id)?;
    
    // Program/timestamp line
    writeln!(writer, "  DrugCandidateStudio")?;
    
    // Comment line
    writeln!(writer, "")?;
    
    // Counts line (simplified - no actual atom/bond counts)
    // In real SDF, this would contain actual molecular structure
    writeln!(writer, "  0  0  0  0  0  0  0  0  0  0999 V2000")?;
    
    // M  END marker
    writeln!(writer, "M  END")?;
    
    // Properties
    writeln!(writer, ">  <SMILES>")?;
    writeln!(writer, "{}", candidate.smiles)?;
    writeln!(writer, "")?;
    
    writeln!(writer, ">  <ID>")?;
    writeln!(writer, "{}", candidate.id)?;
    writeln!(writer, "")?;
    
    writeln!(writer, ">  <Efficacy>")?;
    writeln!(writer, "{:.4}", candidate.efficacy)?;
    writeln!(writer, "")?;
    
    writeln!(writer, ">  <Toxicity>")?;
    writeln!(writer, "{:.4}", candidate.toxicity)?;
    writeln!(writer, "")?;
    
    writeln!(writer, ">  <SynthesisCost>")?;
    writeln!(writer, "{:.4}", candidate.synthesis_cost)?;
    writeln!(writer, "")?;
    
    writeln!(writer, ">  <ManufacturingCost>")?;
    writeln!(writer, "{:.4}", candidate.manufacturing_cost)?;
    writeln!(writer, "")?;
    
    writeln!(writer, ">  <Pareto>")?;
    writeln!(writer, "{}", if candidate.pareto { "1" } else { "0" })?;
    writeln!(writer, "")?;
    
    // Record separator
    writeln!(writer, "$$$$")?;
    
    Ok(())
}

/// Export to simple SMILES file with properties
pub fn export_smiles_file(candidates: &[Candidate], path: &str) -> Result<(), String> {
    let mut file = std::fs::File::create(path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    // Header
    writeln!(file, "# SMILES\tID\tEfficacy\tToxicity\tSynthCost\tMfgCost\tPareto")
        .map_err(|e| format!("Write error: {}", e))?;
    
    for c in candidates {
        writeln!(
            file, 
            "{}\t{}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{}",
            c.smiles, c.id, c.efficacy, c.toxicity, 
            c.synthesis_cost, c.manufacturing_cost,
            if c.pareto { "1" } else { "0" }
        ).map_err(|e| format!("Write error: {}", e))?;
    }
    
    Ok(())
}

/// Parse SDF file and extract SMILES from properties
pub fn import_sdf_file(path: &str, start_id: usize) -> Result<Vec<Candidate>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let mut candidates = Vec::new();
    let mut id = start_id;
    
    // Split by $$$$ record separator
    for record in content.split("$$$$") {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }
        
        // Try to find SMILES property
        if let Some(smiles) = extract_sdf_property(record, "SMILES") {
            let candidate = create_candidate_from_smiles(id, &smiles);
            candidates.push(candidate);
            id += 1;
        }
    }
    
    Ok(candidates)
}

fn extract_sdf_property(record: &str, property: &str) -> Option<String> {
    let pattern = format!(">  <{}>", property);
    
    if let Some(pos) = record.find(&pattern) {
        let start = pos + pattern.len();
        let rest = &record[start..];
        
        // Skip whitespace and get next non-empty line
        for line in rest.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('>') {
                return Some(line.to_string());
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_smiles_text() {
        let text = "CCO\nCCCC\nc1ccccc1";
        let candidates = import_smiles_text(text, 0);
        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].smiles, "CCO");
    }

    #[test]
    fn test_create_candidate() {
        let c = create_candidate_from_smiles(0, "CCO");
        assert!(!c.smiles.is_empty());
        assert!(c.efficacy >= 0.0 && c.efficacy <= 1.0);
    }
}
