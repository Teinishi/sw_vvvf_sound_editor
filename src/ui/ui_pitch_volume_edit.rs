use crate::{
    app_action::AppAction,
    state::{AudioEntry, AudioEntryId},
    ui::{PlotAutoColor, UiFunctionEdit},
};
use egui::{Button, Label, vec2};

#[derive(Debug, Default)]
pub struct UiPitchVolumeEdit;

impl UiPitchVolumeEdit {
    pub const TITLE: &str = "Point Edit";

    #[expect(clippy::unused_self)]
    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        action: &mut AppAction,
        entries: &mut [AudioEntry],
        selection: &mut Option<AudioEntryId>,
    ) {
        ui.strong(Self::TITLE);
        Self::ui_legend(ui, action, entries, selection);

        ui.separator();

        if let Some(entry) = selection
            .as_ref()
            .and_then(|id| entries.iter_mut().find(|e| e.id() == id))
        {
            UiFunctionEdit::new("Pitch", ("Speed", "Pitch")).ui(
                ui,
                ui.id().with("pitch"),
                entry.pitch_mut(),
            );

            ui.separator();

            UiFunctionEdit::new("Volume", ("Speed", "Volume"))
                .y_percentage(true)
                .ui(ui, ui.id().with("volume"), entry.volume_mut());
        }
    }

    fn ui_legend(
        ui: &mut egui::Ui,
        action: &mut AppAction,
        entries: &[AudioEntry],
        selection: &mut Option<AudioEntryId>,
    ) {
        for (index, entry) in entries.iter().enumerate() {
            let color = PlotAutoColor::get_color(index);

            ui.horizontal(|ui| {
                let interact_size = ui.spacing().interact_size;
                ui.add_sized(vec2(20.0, interact_size.y), Label::new(format!("{index}")));

                let (_, stroke_rect) = ui.allocate_space(interact_size);
                ui.painter().line_segment(
                    [stroke_rect.left_center(), stroke_rect.right_center()],
                    (4.0, color),
                );

                let checked = selection
                    .as_ref()
                    .map(|id| id == entry.id())
                    .unwrap_or(false);

                if ui
                    .add_sized(
                        ui.available_size_before_wrap(),
                        Button::selectable(checked, (entry.name().to_string(), egui::Atom::grow()))
                            .truncate(),
                    )
                    .clicked()
                {
                    if checked {
                        *selection = None;
                    } else {
                        *selection = Some(*entry.id());
                    }
                    action.add_undo();
                }
            });
        }
    }
}
