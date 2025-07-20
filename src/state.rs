use crate::func_edit::{EditableFunc, EditablePositiveFunc, EditableZeroOneFunc, FuncEdit as _};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct State {
    pub audio_entries: Vec<AudioEntry>,
    pub selection: Option<AudioEntryId>,
    pub train_performance: TrainPerformance,
    pub speed_cursor: Cursor, // 今のところ不使用
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

pub type AudioEntryId = PathBuf;

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AudioEntry {
    path: PathBuf,
    volume_function: EditableZeroOneFunc,
    pitch_function: EditablePositiveFunc,
}

impl AudioEntry {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            volume_function: EditableFunc::with_points(vec![(40.0, 0.5)]).into(),
            pitch_function: EditableFunc::with_points(vec![(40.0, 1.0)]).into(),
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TrainPerformance {
    pub acceleration: EditablePositiveFunc,
    pub power_steps: u8,
    pub brake_acceleration: f64,
    pub brake_steps: u8,
    pub drag: EditablePositiveFunc,
}

impl Default for TrainPerformance {
    fn default() -> Self {
        Self {
            acceleration: EditableFunc::with_expression("min(2.5,90/x,(80/x)^2)").into(),
            power_steps: 5,
            brake_acceleration: 4.2,
            brake_steps: 8,
            drag: EditableFunc::with_expression("x/500").into(),
        }
    }
}

impl TrainPerformance {
    pub fn update(&mut self) {
        self.acceleration.update_expression();
        self.drag.update_expression();
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    values: Option<(f64, f64)>,
}

impl Cursor {
    pub fn range(&self) -> Option<(f64, f64)> {
        if let Some((a, b)) = self.values {
            Some((a.min(b), a.max(b)))
        } else {
            None
        }
    }

    pub fn set_spot(&mut self, value: f64) {
        self.values = Some((value, value));
    }

    pub fn extend(&mut self, value: f64) {
        if let Some((_, b)) = self.values.as_mut() {
            *b = value;
        } else {
            self.set_spot(value);
        }
    }
}
