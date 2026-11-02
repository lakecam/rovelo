use crate::model::Diagram;

/// Snapshot undo/redo. Each user intent calls `commit` once, so one gesture is one step.
/// Diagrams are small, so whole-document snapshots beat a command inverse for each edit.
#[derive(Default)]
pub struct History {
    past: Vec<Diagram>,
    future: Vec<Diagram>,
}

const LIMIT: usize = 200;

impl History {
    /// Record `before` (the state prior to an edit) so it can be restored.
    pub fn commit(&mut self, before: Diagram) {
        self.past.push(before);
        if self.past.len() > LIMIT {
            self.past.remove(0);
        }
        self.future.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn undo(&mut self, current: &Diagram) -> Option<Diagram> {
        let prev = self.past.pop()?;
        self.future.push(current.clone());
        Some(prev)
    }

    pub fn redo(&mut self, current: &Diagram) -> Option<Diagram> {
        let next = self.future.pop()?;
        self.past.push(current.clone());
        Some(next)
    }

    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_redo_walk() {
        let mut h = History::default();
        let mut d = Diagram::default();
        h.commit(d.clone());
        d.title = "One".into();
        h.commit(d.clone());
        d.title = "Two".into();
        d = h.undo(&d).unwrap();
        assert_eq!(d.title, "One");
        d = h.undo(&d).unwrap();
        assert_eq!(d.title, "Untitled");
        assert!(h.undo(&d).is_none());
        d = h.redo(&d).unwrap();
        assert_eq!(d.title, "One");
        d = h.redo(&d).unwrap();
        assert_eq!(d.title, "Two");
    }

    #[test]
    fn new_edit_clears_redo() {
        let mut h = History::default();
        let d = Diagram::default();
        h.commit(d.clone());
        let d2 = h.undo(&d).unwrap();
        assert!(h.can_redo());
        h.commit(d2);
        assert!(!h.can_redo());
    }
}
