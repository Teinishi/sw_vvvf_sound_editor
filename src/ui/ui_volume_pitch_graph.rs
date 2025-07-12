use egui::{Color32, Pos2, Rect, Sense, Shape, Vec2, emath, epaint::PathStroke, pos2};

#[derive(Debug)]
pub struct UiVolumePitchGraph {
    points: Vec<Pos2>,
}

impl Default for UiVolumePitchGraph {
    fn default() -> Self {
        Self {
            points: vec![pos2(10.0, 10.0), pos2(100.0, 100.0)],
        }
    }
}

impl UiVolumePitchGraph {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let point_radius = 5.0;

        let control_point_shapes: Vec<Shape> = self
            .points
            .iter_mut()
            .enumerate()
            .map(|(i, point)| {
                let size = Vec2::splat(2.0 * point_radius);

                let point_in_screen = to_screen.transform_pos(*point);
                let point_rect = Rect::from_center_size(point_in_screen, size);
                let point_id = response.id.with(i);
                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                *point += point_response.drag_delta();
                *point = to_screen.from().clamp(*point);

                let point_in_screen = to_screen.transform_pos(*point);
                let stroke = ui.style().interact(&point_response).fg_stroke;

                Shape::circle_stroke(point_in_screen, point_radius, stroke)
            })
            .collect();

        let points_in_screen: Vec<Pos2> = self.points.iter().map(|p| to_screen * *p).collect();

        let lines = Shape::line(points_in_screen, PathStroke::new(1.0, Color32::GREEN));

        painter.add(lines);
        painter.extend(control_point_shapes);
    }
}
