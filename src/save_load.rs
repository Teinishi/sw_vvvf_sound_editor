use crate::{
    func_edit::{EditablePositiveFunc, EditableZeroOneFunc},
    state::{AudioEntry, AudioEntryId, FileRegistory, State, TrainPerformance},
};
use anyhow::bail;
use std::{fs::File, io::Write as _, path::PathBuf};
use zip::{ZipWriter, write::SimpleFileOptions};

#[derive(serde::Serialize)]
struct SavedState<'a> {
    audio_entries: Vec<SavedAudioEntry<'a>>,
    train_performance: &'a TrainPerformance,
}

impl<'a> From<&'a State> for SavedState<'a> {
    fn from(value: &'a State) -> Self {
        Self {
            audio_entries: value.audio_entries.iter().map(|e| e.into()).collect(),
            train_performance: &value.train_performance,
        }
    }
}

#[derive(serde::Serialize)]
struct SavedAudioEntry<'a> {
    #[serde(skip)]
    id: AudioEntryId,
    audio: String,
    name: &'a str,
    volume_function: &'a EditableZeroOneFunc,
    pitch_function: &'a EditablePositiveFunc,
}

impl<'a> From<&'a AudioEntry> for SavedAudioEntry<'a> {
    fn from(value: &'a AudioEntry) -> Self {
        let id = *value.id();
        Self {
            id,
            audio: format!("{id}.ogg"),
            name: value.name(),
            volume_function: value.volume(),
            pitch_function: value.pitch(),
        }
    }
}

pub fn save_file(
    path: PathBuf,
    registory: &FileRegistory,
    state: &State,
    state_filepath: &mut Option<PathBuf>,
) -> anyhow::Result<()> {
    let saved_state: SavedState<'_> = state.into();
    let json = serde_json::to_string(&saved_state)?;
    let file = File::create(&path)?;
    let mut zip = ZipWriter::new(file);
    let zip_options = SimpleFileOptions::default();

    // ファイルフォーマットのバージョン
    zip.start_file("VERSION", zip_options)?;
    zip.write_all(b"0")?;

    // JSON
    zip.start_file("state.json", zip_options)?;
    zip.write_all(json.as_bytes())?;

    // 音声ファイル
    for entry in &saved_state.audio_entries {
        if let Some(bytes) = registory.raw_data_by_id(&entry.id) {
            zip.start_file(&entry.audio, zip_options)?;
            zip.write_all(bytes)?;
        } else {
            bail!("Unable to get audio data for {}", entry.name);
        }
    }

    zip.finish()?;

    *state_filepath = Some(path);
    Ok(())
}
