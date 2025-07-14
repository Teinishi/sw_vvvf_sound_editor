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
            volume_function: EditableFunction {
                points: vec![(40.0, 0.5)],
                bounds: Bounds::new(Some(0.0), None, Some(0.0), Some(1.0)),
            },
            pitch_function: EditableFunction {
                points: vec![(40.0, 1.0)],
                bounds: Bounds::new(Some(0.0), None, Some(0.0), None),
            },
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

    pub fn volume(&self) -> &EditableFunction {
        &self.volume_function
    }

    pub fn pitch(&self) -> &EditableFunction {
        &self.pitch_function
    }

    pub fn volume_mut(&mut self) -> &mut EditableFunction {
        &mut self.volume_function
    }

    pub fn pitch_mut(&mut self) -> &mut EditableFunction {
        &mut self.pitch_function
    }

    pub fn volume_at(&self, x: f64) -> f64 {
        self.volume_function.value_at(x)
    }

    pub fn pitch_at(&self, x: f64) -> f64 {
        self.pitch_function.value_at(x)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct EditableFunction {
    points: Vec<(f64, f64)>,
    bounds: Bounds,
}

impl Default for EditableFunction {
    fn default() -> Self {
        Self {
            points: vec![(0.0, 0.0)],
            bounds: Bounds::default(),
        }
    }
}

impl EditableFunction {
    pub fn points(&self) -> &Vec<(f64, f64)> {
        &self.points
    }

    fn find_segment(&self, x: f64) -> usize {
        let p0 = self.points[0];
        if x < p0.0 {
            return 0;
        }
        for (i, p) in self.points.windows(2).enumerate() {
            if p[0].0 <= x && x < p[1].0 {
                return i + 1;
            }
        }
        self.points.len()
    }

    pub fn value_at(&self, x: f64) -> f64 {
        let i = self.find_segment(x);
        if i == 0 {
            if let Some(first2) = self.points.first_chunk::<2>() {
                self.bounds
                    .clamp_y(Self::value_at_line(first2[0], first2[1], x))
            } else {
                self.points.first().map(|p| p.1).unwrap_or(0.0)
            }
        } else if i >= self.points.len() {
            if let Some(last2) = self.points.last_chunk::<2>() {
                self.bounds
                    .clamp_y(Self::value_at_line(last2[0], last2[1], x))
            } else {
                self.points.last().map(|p| p.1).unwrap_or(0.0)
            }
        } else {
            Self::value_at_line(self.points[i - 1], self.points[i], x)
        }
    }

    pub fn insert_point(&mut self, point: (f64, f64)) {
        self.points
            .insert(self.find_segment(point.0), self.bounds.clamp(point));
    }

    pub fn split_segment(&mut self, x: f64) {
        self.insert_point((x, self.value_at(x)));
    }

    pub fn remove_point(&mut self, index: usize) {
        if self.points.len() >= 2 {
            self.points.remove(index);
        }
    }

    pub fn move_point(&mut self, index: usize, delta: (f64, f64)) {
        let left = index
            .checked_sub(1)
            .and_then(|l| self.points.get(l).map(|p| p.0));
        let right = index
            .checked_add(1)
            .and_then(|l| self.points.get(l).map(|p| p.0));
        if let Some(point) = self.points.get_mut(index) {
            point.0 += delta.0;
            point.1 += delta.1;
            if let Some(left) = left {
                point.0 = point.0.max(left);
            }
            if let Some(right) = right {
                point.0 = point.0.min(right);
            }
            *point = self.bounds.clamp(*point);
        }
    }

    fn value_at_line(p0: (f64, f64), p1: (f64, f64), x: f64) -> f64 {
        let (x0, y0) = p0;
        let (x1, y1) = p1;
        if x0 == x1 {
            if (y1 - y0) * (x - x0) > 0.0 {
                return f64::INFINITY;
            } else {
                return f64::NEG_INFINITY;
            }
        }
        (y1 - y0) / (x1 - x0) * (x - x0) + y0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
struct Bounds {
    min_x: Option<f64>,
    max_x: Option<f64>,
    min_y: Option<f64>,
    max_y: Option<f64>,
}

impl Bounds {
    fn new(min_x: Option<f64>, max_x: Option<f64>, min_y: Option<f64>, max_y: Option<f64>) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    fn clamp_x(&self, mut x: f64) -> f64 {
        if let Some(max) = self.max_x {
            x = x.min(max);
        }
        if let Some(min) = self.min_x {
            x = x.max(min);
        }
        x
    }

    fn clamp_y(&self, mut y: f64) -> f64 {
        if let Some(max) = self.max_y {
            y = y.min(max);
        }
        if let Some(min) = self.min_y {
            y = y.max(min);
        }
        y
    }

    fn clamp(&self, point: (f64, f64)) -> (f64, f64) {
        (self.clamp_x(point.0), self.clamp_y(point.1))
    }
}
