use super::AudioEntryId;
use crate::func_edit::{EditableFunc, EditablePositiveFunc, EditableZeroOneFunc};
use anyhow::Context as _;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AudioEntry {
    #[serde(skip)]
    pub(super) id: AudioEntryId,
    pub(super) path: Option<PathBuf>,
    name: String,
    volume_function: EditableZeroOneFunc,
    pitch_function: EditablePositiveFunc,
}

impl AudioEntry {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
            path: None,
            volume_function: EditableFunc::with_points(vec![(40.0, 0.5)]).into(),
            pitch_function: EditableFunc::with_points(vec![(40.0, 1.0)]).into(),
        }
    }

    pub fn with_path(id: u32, path: PathBuf) -> anyhow::Result<Self> {
        let without_extension = path.with_extension("");
        let name = without_extension
            .file_name()
            .and_then(|n| n.to_str())
            .context("Unable to get name of file")?;
        Ok(Self {
            id,
            name: name.to_owned(),
            path: Some(path),
            volume_function: EditableFunc::with_points(vec![(40.0, 0.5)]).into(),
            pitch_function: EditableFunc::with_points(vec![(40.0, 1.0)]).into(),
        })
    }

    pub fn id(&self) -> &AudioEntryId {
        &self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn volume(&self) -> &EditableZeroOneFunc {
        &self.volume_function
    }

    pub fn pitch(&self) -> &EditablePositiveFunc {
        &self.pitch_function
    }

    pub fn volume_mut(&mut self) -> &mut EditableZeroOneFunc {
        &mut self.volume_function
    }

    pub fn pitch_mut(&mut self) -> &mut EditablePositiveFunc {
        &mut self.pitch_function
    }
}
