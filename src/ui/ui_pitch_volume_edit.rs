use crate::{
    app_action::AppAction,
    state::{AudioEntry, AudioEntryId, AudioFunctionMode, AudioFunctions},
    ui::{PlotAutoColor, UiFunctionEdit},
};
use egui::{Button, ComboBox, Label, vec2};

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
            ComboBox::from_label("Mode")
                .selected_text(entry.mode().label_text())
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(entry.mode().is_common(), AudioFunctionMode::TEXT_COMMON)
                        .clicked()
                    {
                        *entry.mode_mut() = entry.mode().to_common();
                        action.add_undo();
                    }
                    if ui
                        .selectable_label(
                            entry.mode().is_separate(),
                            AudioFunctionMode::TEXT_SEPARATE,
                        )
                        .clicked()
                    {
                        *entry.mode_mut() = entry.mode().to_separate();
                        action.add_undo();
                    }
                    if ui
                        .selectable_label(
                            entry.mode().is_accel_only(),
                            AudioFunctionMode::TEXT_ACCEL_ONLY,
                        )
                        .clicked()
                    {
                        *entry.mode_mut() = entry.mode().to_accel_only();
                        action.add_undo();
                    }
                    if ui
                        .selectable_label(
                            entry.mode().is_brake_only(),
                            AudioFunctionMode::TEXT_BRAKE_ONLY,
                        )
                        .clicked()
                    {
                        *entry.mode_mut() = entry.mode().to_brake_only();
                        action.add_undo();
                    }
                });

            if let AudioFunctionMode::Common(funcs) = entry.mode_mut() {
                Self::ui_funcs(ui, funcs, "");
            } else {
                if let Some(funcs) = entry.mode_mut().accel_mut() {
                    Self::ui_funcs(ui, funcs, "Accel ");
                }
                if let Some(funcs) = entry.mode_mut().brake_mut() {
                    Self::ui_funcs(ui, funcs, "Brake ");
                }
            }
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

    fn ui_funcs(ui: &mut egui::Ui, funcs: &mut AudioFunctions, title_prefix: &str) {
        let title_pitch = format!("{title_prefix}Pitch");
        let title_volume = format!("{title_prefix}Volume");
        UiFunctionEdit::new(&title_pitch, ("Speed", "Pitch")).ui(
            ui,
            ui.id().with(&title_pitch),
            &mut funcs.pitch,
        );
        ui.add_space(10.0);
        UiFunctionEdit::new(&title_volume, ("Speed", "Volume"))
            .y_percentage(true)
            .ui(ui, ui.id().with(&title_volume), &mut funcs.volume);
    }
}
