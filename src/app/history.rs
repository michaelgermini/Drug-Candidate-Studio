//! Undo/Redo history management

use super::state::Candidate;
use serde::{Serialize, Deserialize};

/// Action types that can be undone/redone
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Generate { candidates: Vec<Candidate> },
    Clear { candidates: Vec<Candidate> },
    Import { candidates: Vec<Candidate> },
    Delete { candidate: Candidate },
    UpdateAnnotation { id: usize, old_note: Option<String>, new_note: Option<String> },
    ToggleFavorite { id: usize },
}

/// History manager for undo/redo
#[derive(Clone, Debug, Default)]
pub struct History {
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
    max_history: usize,
}

impl History {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Record a new action (clears redo stack)
    pub fn push(&mut self, action: Action) {
        self.redo_stack.clear();
        self.undo_stack.push(action);
        
        // Trim if exceeds max
        while self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last action, returns the action if available
    pub fn undo(&mut self) -> Option<Action> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    /// Redo the last undone action
    pub fn redo(&mut self) -> Option<Action> {
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get number of actions in undo stack
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get number of actions in redo stack
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get description of last action for undo
    pub fn last_action_description(&self) -> Option<String> {
        self.undo_stack.last().map(|a| match a {
            Action::Generate { candidates } => format!("Generate {} candidates", candidates.len()),
            Action::Clear { candidates } => format!("Clear {} candidates", candidates.len()),
            Action::Import { candidates } => format!("Import {} candidates", candidates.len()),
            Action::Delete { candidate } => format!("Delete candidate {}", candidate.id),
            Action::UpdateAnnotation { id, .. } => format!("Update annotation for #{}", id),
            Action::ToggleFavorite { id } => format!("Toggle favorite for #{}", id),
        })
    }
}

/// Annotations storage
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Annotations {
    notes: std::collections::HashMap<usize, String>,
    favorites: std::collections::HashSet<usize>,
}

impl Annotations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_note(&mut self, id: usize, note: String) {
        if note.is_empty() {
            self.notes.remove(&id);
        } else {
            self.notes.insert(id, note);
        }
    }

    pub fn get_note(&self, id: usize) -> Option<&String> {
        self.notes.get(&id)
    }

    pub fn toggle_favorite(&mut self, id: usize) -> bool {
        if self.favorites.contains(&id) {
            self.favorites.remove(&id);
            false
        } else {
            self.favorites.insert(id);
            true
        }
    }

    pub fn is_favorite(&self, id: usize) -> bool {
        self.favorites.contains(&id)
    }

    pub fn favorite_count(&self) -> usize {
        self.favorites.len()
    }

    pub fn get_favorites(&self) -> Vec<usize> {
        self.favorites.iter().cloned().collect()
    }

    pub fn notes_count(&self) -> usize {
        self.notes.len()
    }

    pub fn clear(&mut self) {
        self.notes.clear();
        self.favorites.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(id: usize) -> Candidate {
        Candidate {
            id,
            smiles: format!("C{}", id),
            efficacy: 0.5,
            toxicity: 0.3,
            synthesis_cost: 0.2,
            manufacturing_cost: 0.2,
            pareto: false,
        }
    }

    #[test]
    fn test_undo_redo() {
        let mut history = History::new(10);
        
        history.push(Action::Generate { 
            candidates: vec![make_candidate(0)] 
        });
        
        assert!(history.can_undo());
        assert!(!history.can_redo());
        
        let action = history.undo();
        assert!(action.is_some());
        assert!(!history.can_undo());
        assert!(history.can_redo());
        
        let action = history.redo();
        assert!(action.is_some());
        assert!(history.can_undo());
    }

    #[test]
    fn test_annotations() {
        let mut annotations = Annotations::new();
        
        annotations.set_note(1, "Test note".to_string());
        assert_eq!(annotations.get_note(1), Some(&"Test note".to_string()));
        
        annotations.toggle_favorite(1);
        assert!(annotations.is_favorite(1));
        
        annotations.toggle_favorite(1);
        assert!(!annotations.is_favorite(1));
    }
}
