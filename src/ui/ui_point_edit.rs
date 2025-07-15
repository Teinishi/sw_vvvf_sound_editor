use egui::{Button, DragValue, Label, RichText, vec2};

use crate::{
    state::{AudioEntry, AudioEntryId, EditableFunction},
    ui::PlotAutoColor,
};

#[derive(Debug, Default)]
pub struct UiPointEdit;

impl UiPointEdit {
    #[expect(clippy::unused_self)]
    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        entries: &mut [AudioEntry],
        selection: &mut Option<AudioEntryId>,
    ) {
        ui.strong("Point Edit");
        Self::ui_legend(ui, entries, selection);

        ui.separator();

        if let Some(entry) = selection
            .as_ref()
            .and_then(|path| entries.iter_mut().find(|e| e.path() == path))
        {
            ui.horizontal(|ui| {
                ui.add_sized(
                    vec2(60.0, 16.0),
                    Label::new(RichText::new("Speed").strong()),
                );
                ui.add_sized(
                    vec2(60.0, 16.0),
                    Label::new(RichText::new("Pitch").strong()),
                );
            });
            Self::ui_points(ui, entry.pitch_mut(), false);

            ui.separator();

            ui.horizontal(|ui| {
                ui.add_sized(
                    vec2(60.0, 16.0),
                    Label::new(RichText::new("Speed").strong()),
                );
                ui.add_sized(
                    vec2(60.0, 16.0),
                    Label::new(RichText::new("Volume").strong()),
                );
            });
            Self::ui_points(ui, entry.volume_mut(), true);

            /*ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
                if let Some(path) = entry.path().to_str() {
                    ui.weak(path);
                }
            });*/
        }
    }

    fn ui_legend(ui: &mut egui::Ui, entries: &[AudioEntry], selection: &mut Option<AudioEntryId>) {
        for (index, entry) in entries.iter().enumerate() {
            let color = PlotAutoColor::get_color(index);

            ui.horizontal(|ui| {
                let (_, stroke_rect) = ui.allocate_space(ui.spacing().interact_size);
                ui.painter().line_segment(
                    [stroke_rect.left_center(), stroke_rect.right_center()],
                    (4.0, color),
                );

                let checked = selection
                    .as_ref()
                    .map(|path| path == entry.path())
                    .unwrap_or(false);

                if ui
                    .add_sized(
                        ui.available_size_before_wrap(),
                        Button::selectable(
                            checked,
                            (entry.name().unwrap_or_default(), egui::Atom::grow()),
                        )
                        .truncate(),
                    )
                    .clicked()
                {
                    if checked {
                        *selection = None;
                    } else {
                        *selection = Some(entry.path().clone());
                    }
                }
            });
        }
    }

    fn ui_points(ui: &mut egui::Ui, func: &mut EditableFunction, is_y_percentage: bool) {
        let mut changed = None;

        for (i, point) in func.points().iter().enumerate() {
            let mut x = point.0;
            let mut y = point.1;

            ui.horizontal(|ui| {
                let drag_x = DragValue::new(&mut x).speed(0.1).max_decimals(1);
                let mut drag_y = DragValue::new(&mut y).speed(0.01).max_decimals(3);
                if is_y_percentage {
                    drag_y = drag_y
                        .custom_formatter(|x, _| format!("{:.1}", 100.0 * x))
                        .custom_parser(|s| s.parse().ok().map(|v: f64| v / 100.0))
                        .suffix("%");
                }
                let changed_x = ui.add_sized(vec2(60.0, 20.0), drag_x).changed();
                let changed_y = ui.add_sized(vec2(60.0, 20.0), drag_y).changed();
                if changed_x || changed_y {
                    changed = Some((i, (x, y)));
                }
            });
        }

        if let Some((i, pos)) = changed {
            func.move_point_to(i, pos);
        }
    }
}
