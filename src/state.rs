use std::path::PathBuf;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct State {
    pub audio_entries: Vec<AudioEntry>,
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
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct AudioEntry {
    path: PathBuf,
    volume_function: EditableFunction,
    pitch_function: EditableFunction,
}

impl AudioEntry {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            volume_function: Default::default(),
            pitch_function: Default::default(),
        }
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
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct EditableFunction {
    points: Vec<(f32, f32)>,
}

impl Default for EditableFunction {
    fn default() -> Self {
        Self {
            points: vec![(0.0, 1.0)],
        }
    }
}
