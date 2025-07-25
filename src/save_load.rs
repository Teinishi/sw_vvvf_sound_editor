use crate::{
    func_edit::{EditablePositiveFunc, EditableZeroOneFunc},
    state::{AudioEntry, AudioEntryId, FileRegistory, State, TrainPerformance},
};
use anyhow::bail;
use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

#[derive(serde::Serialize)]
struct SerializeState<'a> {
    audio_entries: Vec<SerializeAudioEntry<'a>>,
    train_performance: &'a TrainPerformance,
}

impl<'a> From<&'a State> for SerializeState<'a> {
    fn from(value: &'a State) -> Self {
        Self {
            audio_entries: value.audio_entries.iter().map(|e| e.into()).collect(),
            train_performance: &value.train_performance,
        }
    }
}

#[derive(serde::Serialize)]
struct SerializeAudioEntry<'a> {
    #[serde(skip)]
    id: AudioEntryId,
    audio: String,
    name: &'a str,
    volume_function: &'a EditableZeroOneFunc,
    pitch_function: &'a EditablePositiveFunc,
}

impl<'a> From<&'a AudioEntry> for SerializeAudioEntry<'a> {
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

#[derive(serde::Deserialize)]
struct DeserializeState {
    audio_entries: Vec<DeserializeAudioEntry>,
    train_performance: TrainPerformance,
}

#[derive(serde::Deserialize)]
struct DeserializeAudioEntry {
    audio: String,
    name: String,
    volume_function: EditableZeroOneFunc,
    pitch_function: EditablePositiveFunc,
}

pub fn save_file(
    path: PathBuf,
    registory: &FileRegistory,
    state: &State,
    state_filepath: &mut Option<PathBuf>,
) -> anyhow::Result<()> {
    // jsonとoggをまとめてzipにして保存、拡張子だけswvf
    let saved_state: SerializeState<'_> = state.into();
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

pub fn open_file(
    path: PathBuf,
    registory: &mut FileRegistory,
    state: &mut State,
    state_filepath: &mut Option<PathBuf>,
) -> anyhow::Result<()> {
    let file = File::open(&path)?;
    let mut zip = ZipArchive::new(file)?;

    let mut version = String::new();
    zip.by_name("VERSION")?.read_to_string(&mut version)?;
    assert_eq!(version.trim(), "0", "Unsupported file version: {version}");

    let mut json = String::new();
    zip.by_name("state.json")?.read_to_string(&mut json)?;
    let saved_state: DeserializeState = serde_json::from_str(&json)?;

    let mut new_state = State {
        train_performance: saved_state.train_performance,
        ..Default::default()
    };

    let mut new_registory = FileRegistory::default();
    for entry in &saved_state.audio_entries {
        let mut buf = Vec::new();
        zip.by_name(&entry.audio)?.read_to_end(&mut buf)?;
        let id = new_registory.add_buffered_file(buf, &entry.name, &mut new_state)?;
        if let Some(new_entry) = new_state.get_audio_entry_mut(&id) {
            *new_entry.volume_mut() = entry.volume_function.clone();
            *new_entry.pitch_mut() = entry.pitch_function.clone();
        }
    }

    registory.patch_keep_output(new_registory);
    *state = new_state;
    *state_filepath = Some(path);

    Ok(())
}
