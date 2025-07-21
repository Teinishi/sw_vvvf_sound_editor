use super::{AudioEntry, AudioEntryId, TrainPerformance};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct State {
    pub audio_entries: Vec<AudioEntry>,
    pub selection: Option<AudioEntryId>,
    pub train_performance: TrainPerformance,
    //pub speed_cursor: Cursor, // 今のところ不使用
}

impl State {
    pub(super) fn add_audio_entry(&mut self, id: AudioEntryId, name: &str) {
        self.audio_entries.push(AudioEntry::new(id, name));
    }
    pub(super) fn add_audio_entry_with_path(
        &mut self,
        id: AudioEntryId,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        self.audio_entries.push(AudioEntry::with_path(id, path)?);
        Ok(())
    }

    pub fn move_audio_entry(&mut self, from_idx: usize, to_idx: usize) {
        let item = self.remove_audio_entry(from_idx);
        self.audio_entries
            .insert(to_idx.min(self.audio_entries.len()), item);
    }

    pub(super) fn remove_audio_entry(&mut self, index: usize) -> AudioEntry {
        self.audio_entries.remove(index)
    }

    pub(super) fn clear_audio_entries(&mut self) {
        self.audio_entries.clear();
    }
}
