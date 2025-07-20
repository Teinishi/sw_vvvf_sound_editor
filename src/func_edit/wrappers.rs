use super::{EditableFunc, EditableFuncMode, FuncEdit};

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EditableZeroOneFunc {
    inner: EditableFunc,
}

impl From<EditableFunc> for EditableZeroOneFunc {
    fn from(value: EditableFunc) -> Self {
        Self::new(value)
    }
}

impl FuncEdit for EditableZeroOneFunc {
    fn mode(&self) -> &EditableFuncMode {
        self.inner.mode()
    }

    fn mode_mut(&mut self) -> &mut EditableFuncMode {
        self.inner.mode_mut()
    }

    fn value_at(&self, x: f64) -> f64 {
        self.inner.value_at_clamped(x, Self::clamp)
    }

    fn checked_value_at(&self, x: f64) -> Option<f64> {
        self.inner.checked_value_at_clamped(x, Self::clamp)
    }

    fn points(&self) -> Option<&Vec<(f64, f64)>> {
        self.inner.points()
    }

    fn insert_point(&mut self, pos: (f64, f64)) -> (usize, (f64, f64)) {
        self.inner.insert_point_clamped(pos, Self::clamp)
    }

    fn insert_point_by_index(&mut self, index: usize) -> (usize, (f64, f64)) {
        self.inner.insert_point_by_index_clamped(index, Self::clamp)
    }

    fn split_segment(&mut self, x: f64) -> (usize, (f64, f64)) {
        self.inner.split_segment_clamped(x, Self::clamp)
    }

    fn move_point_to(&mut self, index: usize, pos: (f64, f64)) -> Option<(f64, f64)> {
        self.inner.move_point_to_clamped(index, pos, Self::clamp)
    }

    fn remove_point(&mut self, index: usize) -> Option<(f64, f64)> {
        self.inner.remove_point(index)
    }

    fn expression_mut(&mut self) -> Option<&mut String> {
        self.inner.expression_mut()
    }

    fn expression_err(&self) -> Option<&meval::Error> {
        self.inner.expression_err()
    }

    fn update_expression(&mut self) {
        self.inner.update_expression();
    }
}

impl EditableZeroOneFunc {
    pub fn new(inner: EditableFunc) -> Self {
        Self { inner }
    }

    fn clamp(point: (f64, f64)) -> (f64, f64) {
        (point.0.max(0.0), point.1.clamp(0.0, 1.0))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EditablePositiveFunc {
    inner: EditableFunc,
}

impl From<EditableFunc> for EditablePositiveFunc {
    fn from(value: EditableFunc) -> Self {
        Self::new(value)
    }
}

impl FuncEdit for EditablePositiveFunc {
    fn mode(&self) -> &EditableFuncMode {
        self.inner.mode()
    }

    fn mode_mut(&mut self) -> &mut EditableFuncMode {
        self.inner.mode_mut()
    }

    fn value_at(&self, x: f64) -> f64 {
        self.inner.value_at_clamped(x, Self::clamp)
    }

    fn checked_value_at(&self, x: f64) -> Option<f64> {
        self.inner.checked_value_at_clamped(x, Self::clamp)
    }

    fn points(&self) -> Option<&Vec<(f64, f64)>> {
        self.inner.points()
    }

    fn insert_point(&mut self, pos: (f64, f64)) -> (usize, (f64, f64)) {
        self.inner.insert_point_clamped(pos, Self::clamp)
    }

    fn insert_point_by_index(&mut self, index: usize) -> (usize, (f64, f64)) {
        self.inner.insert_point_by_index_clamped(index, Self::clamp)
    }

    fn split_segment(&mut self, x: f64) -> (usize, (f64, f64)) {
        self.inner.split_segment_clamped(x, Self::clamp)
    }

    fn move_point_to(&mut self, index: usize, pos: (f64, f64)) -> Option<(f64, f64)> {
        self.inner.move_point_to_clamped(index, pos, Self::clamp)
    }

    fn remove_point(&mut self, index: usize) -> Option<(f64, f64)> {
        self.inner.remove_point(index)
    }

    fn expression_mut(&mut self) -> Option<&mut String> {
        self.inner.expression_mut()
    }

    fn expression_err(&self) -> Option<&meval::Error> {
        self.inner.expression_err()
    }

    fn update_expression(&mut self) {
        self.inner.update_expression();
    }
}

impl EditablePositiveFunc {
    pub fn new(inner: EditableFunc) -> Self {
        Self { inner }
    }

    fn clamp(point: (f64, f64)) -> (f64, f64) {
        (point.0.max(0.0), point.1.max(0.0))
    }
}
