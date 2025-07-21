#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SelectionCursor {
    values: Option<(f64, f64)>,
}

impl SelectionCursor {
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
