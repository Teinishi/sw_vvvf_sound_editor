use crate::editable_function::{Bounds, EditableFunction};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct State {
    pub audio_entries: Vec<AudioEntry>,
    pub selection: Option<AudioEntryId>,
    pub train_performance: TrainPerformance,
    pub speed_cursor: Cursor,
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
            pitch_function: EditableFunction::with_points(vec![(40.0, 1.0)], Bounds::POSITIVE),
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

    pub fn volume_mut(&mut self) -> &mut EditableFunction {
        &mut self.volume_function
    }

    pub fn pitch_mut(&mut self) -> &mut EditableFunction {
        &mut self.pitch_function
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TrainPerformance {
    pub acceleration: EditableFunction,
    pub power_steps: u8,
    pub brake_acceleration: f64,
    pub brake_steps: u8,
    pub drag: EditableFunction,
}

impl Default for TrainPerformance {
    fn default() -> Self {
        Self {
            acceleration: EditableFunction::with_expression(
                "min(2.5,90/x,(80/x)^2)",
                Bounds::POSITIVE,
            ),
            power_steps: 5,
            brake_acceleration: 4.2,
            brake_steps: 8,
            drag: EditableFunction::with_expression("x/500", Bounds::POSITIVE),
        }
    }
}

impl TrainPerformance {
    pub fn update(&mut self) {
        self.acceleration.update();
        self.drag.update();
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
