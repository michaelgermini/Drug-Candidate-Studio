use crate::{generation, optimization};
use serde::{Serialize, Deserialize};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::thread;
use super::history::{History, Annotations, Action};

#[derive(Debug)]
pub enum WorkerMessage {
    GenerateCandidates { n: usize, seed: u64, start_id: usize, parallel: bool },
    CancelGeneration,
    GenerationProgress { current: usize, total: usize },
    GenerationComplete { candidates: Vec<Candidate> },
    GenerationError(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub id: usize,
    pub smiles: String,
    pub efficacy: f32,            // higher better
    pub toxicity: f32,            // lower better
    pub synthesis_cost: f32,      // lower better
    pub manufacturing_cost: f32,  // lower better
    pub pareto: bool,
}

/// Session data for save/load
#[derive(Serialize, Deserialize)]
pub struct SessionData {
    pub candidates: Vec<Candidate>,
    pub next_id: usize,
    pub n_generate: usize,
    pub seed: u64,
    pub w_eff: f32,
    pub w_tox: f32,
    pub w_syn: f32,
    pub w_mfg: f32,
    pub filter_pareto_only: bool,
    #[serde(default)]
    pub annotations: Annotations,
}

pub struct AppState {
    // session
    pub next_id: usize,
    pub candidates: Vec<Candidate>,
    pub selected_id: Option<usize>,

    // generation
    pub n_generate: usize,
    pub seed: u64,
    pub use_parallel: bool,
    pub use_scaffolds: bool,

    // weights (optionnel: score unique pour tri)
    pub w_eff: f32,
    pub w_tox: f32,
    pub w_syn: f32,
    pub w_mfg: f32,

    // filters
    pub filter_pareto_only: bool,
    pub filter_smiles: String,
    pub filter_eff_min: f32,
    pub filter_eff_max: f32,
    pub filter_tox_min: f32,
    pub filter_tox_max: f32,
    pub filter_favorites_only: bool,

    // status
    pub status: String,

    // worker thread communication
    pub worker_sender: Option<Sender<WorkerMessage>>,
    pub worker_receiver: Option<Receiver<WorkerMessage>>,
    pub is_generating: bool,
    pub generation_progress: Option<(usize, usize)>,
    
    // UI state
    pub show_histograms: bool,
    pub show_parallel_coords: bool,
    pub show_3d_plot: bool,
    pub show_heatmap: bool,
    pub show_clustering: bool,
    pub show_druglikeness: bool,
    pub show_similarity_search: bool,

    // History & Annotations
    pub history: History,
    pub annotations: Annotations,
    
    // Theme
    pub theme_changed: bool,
    
    // Import text buffer
    pub import_text: String,
    pub show_import_dialog: bool,
}

impl Default for Candidate {
    fn default() -> Self {
        Self {
            id: 0,
            smiles: "C".into(),
            efficacy: 0.0,
            toxicity: 0.0,
            synthesis_cost: 0.0,
            manufacturing_cost: 0.0,
            pareto: false,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        let (to_worker_sender, to_worker_receiver) = unbounded();
        let (to_main_sender, to_main_receiver) = unbounded();

        // Spawn worker thread
        thread::spawn(move || {
            generation_worker(to_worker_receiver, to_main_sender);
        });

        Self {
            next_id: 0,
            candidates: vec![],
            selected_id: None,
            n_generate: 300,
            seed: 42,
            use_parallel: true,
            use_scaffolds: true,
            w_eff: 1.0,
            w_tox: 1.0,
            w_syn: 1.0,
            w_mfg: 1.0,
            filter_pareto_only: false,
            filter_smiles: String::new(),
            filter_eff_min: 0.0,
            filter_eff_max: 1.0,
            filter_tox_min: 0.0,
            filter_tox_max: 1.0,
            filter_favorites_only: false,
            status: "Ready - Click 'Generate' to start".into(),
            worker_sender: Some(to_worker_sender),
            worker_receiver: Some(to_main_receiver),
            is_generating: false,
            generation_progress: None,
            show_histograms: false,
            show_parallel_coords: false,
            show_3d_plot: false,
            show_heatmap: false,
            show_clustering: false,
            show_druglikeness: true,
            show_similarity_search: false,
            history: History::new(50),
            annotations: Annotations::new(),
            theme_changed: false,
            import_text: String::new(),
            show_import_dialog: false,
        }
    }
}

impl AppState {
    pub fn weighted_score(&self, c: &Candidate) -> f32 {
        self.w_eff * c.efficacy
            - self.w_tox * c.toxicity
            - self.w_syn * c.synthesis_cost
            - self.w_mfg * c.manufacturing_cost
    }

    pub fn generate(&mut self) {
        if self.is_generating {
            return;
        }

        if let Some(sender) = &self.worker_sender {
            self.is_generating = true;
            self.generation_progress = Some((0, self.n_generate));
            let mode = if self.use_parallel { "parallel" } else { "sequential" };
            self.status = format!("Generating {} candidates ({})...", self.n_generate, mode);

            let _ = sender.send(WorkerMessage::GenerateCandidates {
                n: self.n_generate,
                seed: self.seed,
                start_id: self.next_id,
                parallel: self.use_parallel,
            });
        }
    }

    /// Filter candidates based on current filter settings
    pub fn filtered_candidates(&self) -> Vec<&Candidate> {
        self.candidates
            .iter()
            .filter(|c| {
                // Pareto filter
                if self.filter_pareto_only && !c.pareto {
                    return false;
                }
                
                // Favorites filter
                if self.filter_favorites_only && !self.annotations.is_favorite(c.id) {
                    return false;
                }
                
                // SMILES search
                if !self.filter_smiles.is_empty() {
                    let search = self.filter_smiles.to_lowercase();
                    if !c.smiles.to_lowercase().contains(&search) {
                        return false;
                    }
                }
                
                // Efficacy range
                if c.efficacy < self.filter_eff_min || c.efficacy > self.filter_eff_max {
                    return false;
                }
                
                // Toxicity range
                if c.toxicity < self.filter_tox_min || c.toxicity > self.filter_tox_max {
                    return false;
                }
                
                true
            })
            .collect()
    }

    /// Save session to file
    pub fn save_session(&self, path: &str) -> Result<(), String> {
        let session = SessionData {
            candidates: self.candidates.clone(),
            next_id: self.next_id,
            n_generate: self.n_generate,
            seed: self.seed,
            w_eff: self.w_eff,
            w_tox: self.w_tox,
            w_syn: self.w_syn,
            w_mfg: self.w_mfg,
            filter_pareto_only: self.filter_pareto_only,
            annotations: self.annotations.clone(),
        };
        
        let json = serde_json::to_string_pretty(&session)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        std::fs::write(path, json)
            .map_err(|e| format!("Write error: {}", e))?;
        
        Ok(())
    }

    /// Load session from file
    pub fn load_session(&mut self, path: &str) -> Result<(), String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Read error: {}", e))?;
        
        let session: SessionData = serde_json::from_str(&json)
            .map_err(|e| format!("Parse error: {}", e))?;
        
        self.candidates = session.candidates;
        self.next_id = session.next_id;
        self.n_generate = session.n_generate;
        self.seed = session.seed;
        self.w_eff = session.w_eff;
        self.w_tox = session.w_tox;
        self.w_syn = session.w_syn;
        self.w_mfg = session.w_mfg;
        self.filter_pareto_only = session.filter_pareto_only;
        self.annotations = session.annotations;
        self.selected_id = None;
        
        self.recompute_pareto();
        
        Ok(())
    }

    pub fn cancel_generation(&mut self) {
        if let Some(sender) = &self.worker_sender {
            let _ = sender.send(WorkerMessage::CancelGeneration);
            self.is_generating = false;
            self.generation_progress = None;
            self.status = "Generation cancelled".into();
        }
    }

    pub fn process_worker_messages(&mut self) {
        let messages: Vec<WorkerMessage> = if let Some(receiver) = &self.worker_receiver {
            let mut msgs = Vec::new();
            while let Ok(msg) = receiver.try_recv() {
                msgs.push(msg);
            }
            msgs
        } else {
            Vec::new()
        };

        for msg in messages {
            match msg {
                WorkerMessage::GenerationProgress { current, total } => {
                    self.generation_progress = Some((current, total));
                    self.status = format!("Generating... {}/{}", current, total);
                }
                WorkerMessage::GenerationComplete { candidates } => {
                    let count = candidates.len();
                    
                    // Record for undo
                    self.history.push(Action::Generate { 
                        candidates: candidates.clone() 
                    });
                    
                    self.next_id += count;
                    self.candidates.extend(candidates);
                    self.recompute_pareto();
                    self.is_generating = false;
                    self.generation_progress = None;
                    let pareto_count = self.candidates.iter().filter(|c| c.pareto).count();
                    self.status = format!(
                        "Generated {} candidates (total: {}, pareto: {})",
                        count, self.candidates.len(), pareto_count
                    );
                }
                WorkerMessage::GenerationError(error) => {
                    self.is_generating = false;
                    self.generation_progress = None;
                    self.status = format!("Error: {}", error);
                }
                _ => {}
            }
        }
    }

    pub fn clear(&mut self) {
        // Record for undo
        if !self.candidates.is_empty() {
            self.history.push(Action::Clear { 
                candidates: self.candidates.clone() 
            });
        }
        
        self.candidates.clear();
        self.selected_id = None;
        self.next_id = 0;
        self.status = "Cleared all candidates".into();
    }

    pub fn recompute_pareto(&mut self) {
        let front_ids = optimization::pareto::pareto_front_ids(&self.candidates);
        for c in &mut self.candidates {
            c.pareto = front_ids.contains(&c.id);
        }
    }

    /// Undo last action
    pub fn undo(&mut self) {
        if let Some(action) = self.history.undo() {
            match action {
                Action::Generate { candidates } => {
                    // Remove the generated candidates
                    let ids: std::collections::HashSet<usize> = candidates.iter().map(|c| c.id).collect();
                    self.candidates.retain(|c| !ids.contains(&c.id));
                    self.next_id = self.candidates.iter().map(|c| c.id).max().map(|m| m + 1).unwrap_or(0);
                    self.recompute_pareto();
                    self.status = format!("Undone: Generated {} candidates", candidates.len());
                }
                Action::Clear { candidates } => {
                    // Restore cleared candidates
                    self.candidates = candidates;
                    self.next_id = self.candidates.iter().map(|c| c.id).max().map(|m| m + 1).unwrap_or(0);
                    self.recompute_pareto();
                    self.status = "Undone: Clear".into();
                }
                Action::Import { candidates } => {
                    let ids: std::collections::HashSet<usize> = candidates.iter().map(|c| c.id).collect();
                    self.candidates.retain(|c| !ids.contains(&c.id));
                    self.recompute_pareto();
                    self.status = format!("Undone: Import {} candidates", candidates.len());
                }
                Action::Delete { candidate } => {
                    self.candidates.push(candidate);
                    self.recompute_pareto();
                    self.status = "Undone: Delete".into();
                }
                Action::UpdateAnnotation { id, old_note, .. } => {
                    if let Some(note) = old_note {
                        self.annotations.set_note(id, note);
                    } else {
                        self.annotations.set_note(id, String::new());
                    }
                }
                Action::ToggleFavorite { id } => {
                    self.annotations.toggle_favorite(id);
                }
            }
        } else {
            self.status = "Nothing to undo".into();
        }
    }

    /// Redo last undone action
    pub fn redo(&mut self) {
        if let Some(action) = self.history.redo() {
            match action {
                Action::Generate { candidates } => {
                    self.candidates.extend(candidates.clone());
                    self.next_id = self.candidates.iter().map(|c| c.id).max().map(|m| m + 1).unwrap_or(0);
                    self.recompute_pareto();
                    self.status = format!("Redone: Generated {} candidates", candidates.len());
                }
                Action::Clear { .. } => {
                    self.candidates.clear();
                    self.next_id = 0;
                    self.status = "Redone: Clear".into();
                }
                Action::Import { candidates } => {
                    self.candidates.extend(candidates.clone());
                    self.recompute_pareto();
                    self.status = format!("Redone: Import {} candidates", candidates.len());
                }
                Action::Delete { candidate } => {
                    self.candidates.retain(|c| c.id != candidate.id);
                    self.recompute_pareto();
                    self.status = "Redone: Delete".into();
                }
                Action::UpdateAnnotation { id, new_note, .. } => {
                    if let Some(note) = new_note {
                        self.annotations.set_note(id, note);
                    }
                }
                Action::ToggleFavorite { id } => {
                    self.annotations.toggle_favorite(id);
                }
            }
        } else {
            self.status = "Nothing to redo".into();
        }
    }

    /// Import candidates from SMILES text
    pub fn import_from_text(&mut self, text: &str) {
        let candidates = super::io::import_smiles_text(text, self.next_id);
        if !candidates.is_empty() {
            self.history.push(Action::Import { candidates: candidates.clone() });
            let count = candidates.len();
            self.next_id += count;
            self.candidates.extend(candidates);
            self.recompute_pareto();
            self.status = format!("Imported {} candidates", count);
        } else {
            self.status = "No valid SMILES found".into();
        }
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self, id: usize) {
        self.history.push(Action::ToggleFavorite { id });
        self.annotations.toggle_favorite(id);
    }

    /// Set annotation note
    pub fn set_note(&mut self, id: usize, note: String) {
        let old_note = self.annotations.get_note(id).cloned();
        self.history.push(Action::UpdateAnnotation { 
            id, 
            old_note, 
            new_note: Some(note.clone()) 
        });
        self.annotations.set_note(id, note);
    }
}

fn generation_worker(receiver: Receiver<WorkerMessage>, sender: Sender<WorkerMessage>) {
    while let Ok(msg) = receiver.recv() {
        match msg {
            WorkerMessage::GenerateCandidates { n, seed, start_id, parallel } => {
                if parallel {
                    let _ = sender.send(WorkerMessage::GenerationProgress {
                        current: 0,
                        total: n,
                    });

                    let candidates = generation::generator::generate_candidates_parallel(
                        start_id,
                        n,
                        seed,
                    );

                    let _ = sender.send(WorkerMessage::GenerationComplete { candidates });
                } else {
                    let batch_size = 50;
                    let mut candidates = Vec::with_capacity(n);
                    let mut cancelled = false;

                    for batch_start in (0..n).step_by(batch_size) {
                        if let Ok(WorkerMessage::CancelGeneration) = receiver.try_recv() {
                            cancelled = true;
                            break;
                        }

                        let batch_end = (batch_start + batch_size).min(n);
                        let batch_count = batch_end - batch_start;

                        let batch_candidates = generation::generator::generate_candidates(
                            start_id + batch_start,
                            batch_count,
                            seed + batch_start as u64,
                        );

                        candidates.extend(batch_candidates);

                        let _ = sender.send(WorkerMessage::GenerationProgress {
                            current: batch_end,
                            total: n,
                        });

                        std::thread::sleep(std::time::Duration::from_millis(2));
                    }

                    if !cancelled {
                        let _ = sender.send(WorkerMessage::GenerationComplete { candidates });
                    } else {
                        let _ = sender.send(WorkerMessage::GenerationError("Cancelled".into()));
                    }
                }
            }
            WorkerMessage::CancelGeneration => {}
            _ => {}
        }
    }
}
