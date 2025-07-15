use egui::{Button, DragValue, Layout, vec2};

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
            ui.strong("Pitch");
            Self::ui_points(ui, entry.pitch_mut());

            ui.separator();

            ui.strong("Volume");
            Self::ui_points(ui, entry.volume_mut());

            ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
                if let Some(path) = entry.path().to_str() {
                    ui.weak(path);
                }
            });
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

    fn ui_points(ui: &mut egui::Ui, func: &mut EditableFunction) {
        let mut changed = None;

        for (i, point) in func.points().iter().enumerate() {
            let mut x = point.0;
            let mut y = point.1;

            ui.horizontal(|ui| {
                let x_changed = ui
                    .add_sized(vec2(60.0, 20.0), DragValue::new(&mut x).speed(0.1))
                    .changed();
                let y_changed = ui
                    .add_sized(vec2(60.0, 20.0), DragValue::new(&mut y).speed(0.01))
                    .changed();
                if x_changed || y_changed {
                    changed = Some((i, (x, y)));
                }
            });
        }

        if let Some((i, pos)) = changed {
            func.move_point_to(i, pos);
        }
    }
}
