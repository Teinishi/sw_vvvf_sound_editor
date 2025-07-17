use std::str::FromStr as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum EditableFunctionMode {
    Points,
    Expression,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EditableFunction {
    pub mode: EditableFunctionMode,
    points: Vec<(f64, f64)>,
    expression_str: String,
    #[serde(skip)]
    expression: Option<Result<meval::Expr, meval::Error>>,
    bounds: Bounds,
}

impl Default for EditableFunction {
    fn default() -> Self {
        Self {
            mode: EditableFunctionMode::Points,
            points: vec![(0.0, 0.0)],
            expression_str: String::new(),
            expression: None,
            bounds: Bounds::default(),
        }
    }
}

impl EditableFunction {
    pub fn with_points(points: Vec<(f64, f64)>, bounds: Bounds) -> Self {
        Self {
            mode: EditableFunctionMode::Points,
            points,
            expression_str: String::new(),
            expression: None,
            bounds,
        }
    }

    pub fn points(&self) -> &Vec<(f64, f64)> {
        &self.points
    }

    pub fn expression(&self) -> &str {
        &self.expression_str
    }

    pub fn expression_err(&self) -> Option<&meval::Error> {
        self.expression.as_ref().and_then(|e| e.as_ref().err())
    }

    pub fn set_expression(&mut self, expression: &str) {
        self.mode = EditableFunctionMode::Expression;
        if expression == self.expression_str {
            return;
        }
        self.expression_str = expression.to_owned();
        if !expression.is_empty() {
            self.expression = Some(meval::Expr::from_str(expression));
        } else {
            self.expression = None;
        }
    }

    pub fn value_at(&self, x: f64) -> f64 {
        match self.mode {
            EditableFunctionMode::Points => {
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
            EditableFunctionMode::Expression => self
                .expression
                .as_ref()
                .and_then(|e| e.as_ref().ok())
                .and_then(|e| e.eval_with_context(meval::Context::new().var("x", x)).ok())
                .unwrap_or(f64::NAN),
        }
    }

    pub fn insert_point(&mut self, point: (f64, f64)) {
        self.points
            .insert(self.find_segment(point.0), self.bounds.clamp(point));
    }

    pub fn insert_point_by_index(&mut self, index: usize) {
        if index == 0 {
            if let Some(p) = self.points.first_chunk::<2>() {
                let x0 = p[0].0;
                let x1 = p[1].0;
                self.split_segment(x0 - (x1 - x0));
            } else if let Some(p) = self.points.first() {
                self.split_segment(p.0 - p.0.abs() * 0.2);
            }
        } else if index >= self.points.len() {
            if let Some(p) = self.points.last_chunk::<2>() {
                let x0 = p[0].0;
                let x1 = p[1].0;
                self.split_segment(x0 + 2.0 * (x1 - x0));
            } else if let Some(p) = self.points.last() {
                self.split_segment(p.0 + p.0.abs() * 0.2);
            }
        } else if let (Some(p0), Some(p1)) = (self.points.get(index - 1), self.points.get(index)) {
            self.split_segment((p0.0 + p1.0) / 2.0);
        }
    }

    pub fn split_segment(&mut self, x: f64) {
        self.insert_point((x, self.value_at(x)));
    }

    pub fn remove_point(&mut self, index: usize) {
        if self.points.len() >= 2 {
            self.points.remove(index);
        }
    }

    pub fn move_point_to(&mut self, index: usize, mut pos: (f64, f64)) {
        if let Some(left) = index.checked_sub(1).and_then(|l| self.points.get(l)) {
            pos.0 = pos.0.max(left.0);
        }
        if let Some(right) = index.checked_add(1).and_then(|l| self.points.get(l)) {
            pos.0 = pos.0.min(right.0);
        }
        pos = self.bounds.clamp(pos);
        if let Some(point) = self.points.get_mut(index) {
            *point = pos;
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

#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Bounds {
    min_x: Option<f64>,
    max_x: Option<f64>,
    min_y: Option<f64>,
    max_y: Option<f64>,
}

impl Bounds {
    pub fn new(
        min_x: Option<f64>,
        max_x: Option<f64>,
        min_y: Option<f64>,
        max_y: Option<f64>,
    ) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    pub fn clamp_x(&self, mut x: f64) -> f64 {
        if let Some(max) = self.max_x {
            x = x.min(max);
        }
        if let Some(min) = self.min_x {
            x = x.max(min);
        }
        x
    }

    pub fn clamp_y(&self, mut y: f64) -> f64 {
        if let Some(max) = self.max_y {
            y = y.min(max);
        }
        if let Some(min) = self.min_y {
            y = y.max(min);
        }
        y
    }

    pub fn clamp(&self, point: (f64, f64)) -> (f64, f64) {
        (self.clamp_x(point.0), self.clamp_y(point.1))
    }
}
