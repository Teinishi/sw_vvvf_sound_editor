use egui::Button;

use crate::{
    state::{AudioEntry, AudioEntryId},
    ui::PlotAutoColor,
};

#[derive(Debug, Default)]
pub struct UiPointEdit;

impl UiPointEdit {
    #[expect(clippy::unused_self)]
    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        entries: &[AudioEntry],
        selection: &mut Option<AudioEntryId>,
    ) {
        ui.strong("Point Edit");

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

        ui.separator();
    }
}
