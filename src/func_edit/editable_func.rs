use super::FuncEdit;
use core::panic;
use std::str::FromStr as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum EditableFuncMode {
    Points,
    Expression,
}

type FnClamp = fn((f64, f64)) -> (f64, f64);

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EditableFunc {
    pub mode: EditableFuncMode,
    points: Vec<(f64, f64)>,
    pub expression: String,
    #[serde(skip)]
    expr_result: Option<(Result<meval::Expr, meval::Error>, String)>,
}

impl Default for EditableFunc {
    fn default() -> Self {
        Self {
            mode: EditableFuncMode::Points,
            points: vec![(0.0, 0.0)],
            expression: String::new(),
            expr_result: None,
        }
    }
}

impl FuncEdit for EditableFunc {
    fn mode(&self) -> &EditableFuncMode {
        &self.mode
    }

    fn mode_mut(&mut self) -> &mut EditableFuncMode {
        &mut self.mode
    }

    fn points(&self) -> Option<&Vec<(f64, f64)>> {
        Some(&self.points)
    }

    fn value_at(&self, x: f64) -> f64 {
        self.value_at_clamped(x, |p| p)
    }

    fn checked_value_at(&self, x: f64) -> Option<f64> {
        self.checked_value_at_clamped(x, |p| p)
    }

    fn insert_point(&mut self, pos: (f64, f64)) -> (usize, (f64, f64)) {
        self.insert_point_clamped(pos, |p| p)
    }

    fn insert_point_by_index(&mut self, index: usize) -> (usize, (f64, f64)) {
        self.insert_point_by_index_clamped(index, |p| p)
    }

    fn split_segment(&mut self, x: f64) -> (usize, (f64, f64)) {
        self.split_segment_clamped(x, |p| p)
    }

    fn move_point_to(&mut self, index: usize, pos: (f64, f64)) -> Option<(f64, f64)> {
        self.move_point_to_clamped(index, pos, |p| p)
    }

    fn remove_point(&mut self, index: usize) -> Option<(f64, f64)> {
        if self.points.len() >= 2 {
            Some(self.points.remove(index))
        } else {
            None
        }
    }

    fn expression_mut(&mut self) -> Option<&mut String> {
        Some(&mut self.expression)
    }

    fn expression_err(&self) -> Option<&meval::Error> {
        self.expr_result
            .as_ref()
            .and_then(|(e, _)| e.as_ref().err())
    }

    fn update_expression(&mut self) {
        if matches!(self.mode, EditableFuncMode::Expression)
            && (self.expression.is_empty() != self.expr_result.is_none()
                || self
                    .expr_result
                    .as_ref()
                    .is_some_and(|(_, s)| s != &self.expression))
        {
            if !self.expression.is_empty() {
                self.expr_result = Some((
                    meval::Expr::from_str(&self.expression),
                    self.expression.clone(),
                ));
            } else {
                self.expr_result = None;
            }
        }
    }
}

impl EditableFunc {
    pub fn with_points(points: Vec<(f64, f64)>) -> Self {
        Self {
            points,
            ..Default::default()
        }
    }

    pub fn with_expression(expression: &str) -> Self {
        let mut s = Self {
            mode: EditableFuncMode::Expression,
            expression: expression.to_owned(),
            ..Default::default()
        };
        s.update_expression();
        s
    }

    pub(super) fn value_at_clamped(&self, x: f64, clamp: FnClamp) -> f64 {
        match self.mode {
            EditableFuncMode::Points => {
                let i = self.find_segment(x);
                if i == 0 {
                    if let Some(first2) = self.points.first_chunk::<2>() {
                        let y = Self::value_at_line(first2[0], first2[1], x);
                        clamp((x, y)).1
                    } else {
                        self.points.first().map(|p| p.1).unwrap_or(0.0)
                    }
                } else if i >= self.points.len() {
                    if let Some(last2) = self.points.last_chunk::<2>() {
                        let y = Self::value_at_line(last2[0], last2[1], x);
                        clamp((x, y)).1
                    } else {
                        self.points.last().map(|p| p.1).unwrap_or(0.0)
                    }
                } else {
                    Self::value_at_line(self.points[i - 1], self.points[i], x)
                }
            }
            EditableFuncMode::Expression => self
                .expr_result
                .as_ref()
                .and_then(|(e, _)| e.as_ref().ok())
                .and_then(|e| e.eval_with_context(meval::Context::new().var("x", x)).ok())
                .unwrap_or(f64::NAN),
        }
    }

    pub(super) fn checked_value_at_clamped(&self, x: f64, clamp: FnClamp) -> Option<f64> {
        let v = self.value_at_clamped(x, clamp);
        if v.is_finite() { Some(v) } else { None }
    }

    pub(super) fn insert_point_clamped(
        &mut self,
        pos: (f64, f64),
        clamp: FnClamp,
    ) -> (usize, (f64, f64)) {
        let pos = clamp(pos);
        let index = self.find_segment(pos.0);
        self.points.insert(index, pos);
        (index, pos)
    }

    pub(super) fn insert_point_by_index_clamped(
        &mut self,
        index: usize,
        clamp: FnClamp,
    ) -> (usize, (f64, f64)) {
        if index == 0 {
            if let Some(p) = self.points.first_chunk::<2>() {
                let x0 = p[0].0;
                let x1 = p[1].0;
                self.split_segment_clamped(x0 - (x1 - x0), clamp)
            } else if let Some(p) = self.points.first() {
                self.split_segment_clamped(p.0 - p.0.abs() * 0.2, clamp)
            } else {
                panic!("EditableFunc has no point")
            }
        } else if index >= self.points.len() {
            if let Some(p) = self.points.last_chunk::<2>() {
                let x0 = p[0].0;
                let x1 = p[1].0;
                self.split_segment_clamped(x0 + 2.0 * (x1 - x0), clamp)
            } else if let Some(p) = self.points.last() {
                self.split_segment_clamped(p.0 + p.0.abs() * 0.2, clamp)
            } else {
                panic!("EditableFunc has no point")
            }
        } else if let (Some(p0), Some(p1)) = (self.points.get(index - 1), self.points.get(index)) {
            self.split_segment_clamped((p0.0 + p1.0) / 2.0, clamp)
        } else {
            panic!("EditableFunc has no point")
        }
    }

    pub(super) fn split_segment_clamped(&mut self, x: f64, clamp: FnClamp) -> (usize, (f64, f64)) {
        self.insert_point_clamped((x, self.value_at_clamped(x, clamp)), clamp)
    }

    pub(super) fn move_point_to_clamped(
        &mut self,
        index: usize,
        mut pos: (f64, f64),
        clamp: FnClamp,
    ) -> Option<(f64, f64)> {
        if let Some(left) = index.checked_sub(1).and_then(|l| self.points.get(l)) {
            pos.0 = pos.0.max(left.0);
        }
        if let Some(right) = index.checked_add(1).and_then(|l| self.points.get(l)) {
            pos.0 = pos.0.min(right.0);
        }
        pos = clamp(pos);
        if let Some(point) = self.points.get_mut(index) {
            *point = pos;
            Some(pos)
        } else {
            None
        }
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
