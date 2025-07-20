use super::EditableFuncMode;

pub trait FuncEdit {
    fn mode(&self) -> &EditableFuncMode;

    fn mode_mut(&mut self) -> &mut EditableFuncMode;

    fn is_mode_points(&self) -> bool {
        matches!(self.mode(), EditableFuncMode::Points)
    }

    fn points(&self) -> Option<&Vec<(f64, f64)>>;

    fn value_at(&self, x: f64) -> f64;

    fn checked_value_at(&self, x: f64) -> Option<f64>;

    #[expect(dead_code)]
    fn insert_point(&mut self, pos: (f64, f64)) -> (usize, (f64, f64));

    fn insert_point_by_index(&mut self, index: usize) -> (usize, (f64, f64));

    fn split_segment(&mut self, x: f64) -> (usize, (f64, f64));

    fn move_point_to(&mut self, index: usize, pos: (f64, f64)) -> Option<(f64, f64)>;

    fn remove_point(&mut self, index: usize) -> Option<(f64, f64)>;

    fn expression_mut(&mut self) -> Option<&mut String>;

    fn expression_err(&self) -> Option<&meval::Error>;

    fn update_expression(&mut self);
}
