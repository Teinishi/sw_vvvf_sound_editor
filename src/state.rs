use crate::editable_function::{Bounds, EditableFunction};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct State {
    pub audio_entries: Vec<AudioEntry>,
    pub selection: Option<AudioEntryId>,
}

impl State {
    pub fn add_audio_entry(&mut self, path: PathBuf) {
        let entry = AudioEntry::new(path);
        if self
            .audio_entries
            .iter()
            .any(|item| item.path == entry.path)
        {
            return;
        }
        self.audio_entries.push(entry);
    }

    pub fn remove_entry(&mut self, index: usize) -> AudioEntry {
        self.audio_entries.remove(index)
    }

    pub fn move_entry(&mut self, from_idx: usize, to_idx: usize) {
        let item = self.remove_entry(from_idx);
        self.audio_entries
            .insert(to_idx.min(self.audio_entries.len()), item);
    }

    pub fn clear_entries(&mut self) {
        self.audio_entries.clear();
    }

    pub fn volume_entries_mut(
        &mut self,
    ) -> impl Iterator<Item = (AudioEntryId, &mut EditableFunction)> {
        self.audio_entries
            .iter_mut()
            .map(|e| (e.identifier(), e.volume_mut()))
    }

    pub fn pitch_entries_mut(
        &mut self,
    ) -> impl Iterator<Item = (AudioEntryId, &mut EditableFunction)> {
        self.audio_entries
            .iter_mut()
            .map(|e| (e.identifier(), e.pitch_mut()))
    }
}

pub type AudioEntryId = PathBuf;

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AudioEntry {
    path: PathBuf,
    volume_function: EditableFunction,
    pitch_function: EditableFunction,
}

impl AudioEntry {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            volume_function: EditableFunction::with_points(
                vec![(40.0, 0.5)],
                Bounds::new(Some(0.0), None, Some(0.0), Some(1.0)),
            ),
            pitch_function: EditableFunction::with_points(
                vec![(40.0, 1.0)],
                Bounds::new(Some(0.0), None, Some(0.0), None),
            ),
        }
    }

    pub fn identifier(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn name(&self) -> Option<String> {
        self.path
            .with_extension("")
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
    }

    pub fn volume_mut(&mut self) -> &mut EditableFunction {
        &mut self.volume_function
    }

    pub fn pitch_mut(&mut self) -> &mut EditableFunction {
        &mut self.pitch_function
    }
}
