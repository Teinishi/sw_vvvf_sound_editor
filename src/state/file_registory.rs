use super::State;
use crate::{
    audio_player::{AudioOutput, AudioSource, ResampledLoopAudio},
    player_state::PlayerState,
    preference::Preference,
    state::AudioEntry,
};
use anyhow::{anyhow, bail};
use lewton::inside_ogg::OggStreamReader;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub type AudioEntryId = u32;

#[derive(Default)]
pub struct FileRegistory {
    raw_data: HashMap<AudioEntryId, Vec<u8>>,
    audio_sources: Arc<Mutex<HashMap<AudioEntryId, anyhow::Result<ResampledLoopAudio>>>>,
    next_id: AudioEntryId,
    audio_output: AudioOutput,
}

impl FileRegistory {
    pub fn add_file(&mut self, path: PathBuf, state: &mut State) -> anyhow::Result<AudioEntryId> {
        // registoryとstateの両方に同時に追加
        let (raw, result) = self.read_file(&path)?;
        let id = self.generate_id();
        if let Ok(mut sources) = self.audio_sources.lock() {
            self.raw_data.insert(id, raw);
            sources.insert(id, result);
            state.add_audio_entry_with_path(id, path)?;
            Ok(id)
        } else {
            bail!("Failed to get access to mutex audio_sources");
        }
    }

    fn add_existing(
        &mut self,
        path: &PathBuf,
        entry: &mut AudioEntry,
    ) -> anyhow::Result<AudioEntryId> {
        // stateにすでにあるものをregistoryに追加
        let (raw, result) = self.read_file(path)?;
        let new_id = self.generate_id();
        if let Ok(mut sources) = self.audio_sources.lock() {
            self.raw_data.insert(new_id, raw);
            sources.insert(new_id, result);
            entry.id = new_id;
            Ok(new_id)
        } else {
            bail!("Failed to get access to mutex audio_sources");
        }
    }

    pub fn remove_file(&mut self, id: &AudioEntryId, state: &mut State) -> anyhow::Result<()> {
        if let Ok(mut sources) = self.audio_sources.lock() {
            self.raw_data.remove(id);
            sources.remove(id);
            state.audio_entries.retain(|e| e.id() != id);
            Ok(())
        } else {
            bail!("Failed to get access to mutex audio_sources");
        }
    }

    pub fn clear(&mut self, state: &mut State) {
        if let Ok(mut sources) = self.audio_sources.lock() {
            self.raw_data.clear();
            sources.clear();
            state.clear_audio_entries();
        }
    }

    pub fn play_audio(&self, audio_output: &mut AudioOutput) -> anyhow::Result<()> {
        let sources = self.audio_sources.clone();

        audio_output.play(move |data| {
            if let Ok(mut sources) = sources.lock() {
                for entry in sources.values_mut().flatten() {
                    entry.write_data_additive(data);
                }
            }
        })
    }

    pub fn assign_ids(&mut self, state: &mut State) {
        for entry in &mut state.audio_entries {
            entry.id = self.generate_id();
        }
    }

    pub fn update(
        &mut self,
        state: &mut State,
        player_state: &PlayerState,
        preference: &Preference,
    ) -> Vec<anyhow::Error> {
        let mut to_load = vec![];
        let mut indices_to_remove = vec![];
        let mut errors = vec![];

        if let Ok(mut sources) = self.audio_sources.lock() {
            for (index, entry) in state.audio_entries.iter().enumerate() {
                let id = entry.id;
                if let Some(Ok(source)) = sources.get_mut(&id) {
                    let (volume, pitch) = player_state.get_volume_pitch(entry, preference);
                    source.set_volume_pitch(volume, pitch);
                } else if !sources.contains_key(&id) {
                    // 未ロードの音声あり
                    if let Some(path) = entry.path.as_ref() {
                        to_load.push((index, path.clone()));
                    } else {
                        errors.push(anyhow!("Unable to get audio data of {}", entry.name()));
                        indices_to_remove.push(index);
                    }
                }
            }
        }

        indices_to_remove.sort();
        indices_to_remove.reverse();
        for index in indices_to_remove {
            state.remove_audio_entry(index);
        }

        for (index, path) in to_load {
            if let Some(entry) = state.audio_entries.get_mut(index) {
                if let Err(err) = self.add_existing(&path, entry) {
                    errors.push(err);
                }
            }
        }
        errors
    }

    fn generate_id(&mut self) -> AudioEntryId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn read_file(
        &self,
        path: &PathBuf,
    ) -> anyhow::Result<(Vec<u8>, anyhow::Result<ResampledLoopAudio>)> {
        let config = self.audio_output.config()?;
        let raw = std::fs::read(path)?;
        let mut ogg = OggStreamReader::new(std::io::Cursor::new(&raw))?;
        let source = AudioSource::from_ogg(&mut ogg, config.channels() as usize)?;
        let result = ResampledLoopAudio::new(source, config.sample_rate().0, 256);
        Ok((raw, result))
    }
}
