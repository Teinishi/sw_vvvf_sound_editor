use crate::{
    app::AppAction,
    state::{AudioEntry, AudioEntryId},
    ui::{PlotAutoColor, UiFunctionEdit},
};
use egui::{Button, Label, RichText, vec2};

#[derive(Debug, Default)]
pub struct UiPitchVolumeEdit;

impl UiPitchVolumeEdit {
    #[expect(clippy::unused_self)]
    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        action: &mut AppAction,
        entries: &mut [AudioEntry],
        selection: &mut Option<AudioEntryId>,
    ) {
        ui.strong("Point Edit");
        Self::ui_legend(ui, action, entries, selection);

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
            UiFunctionEdit::default().ui(ui, ui.id().with("pitch"), "Pitch", entry.pitch_mut());

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
            UiFunctionEdit::new((false, true)).ui(
                ui,
                ui.id().with("volume"),
                "Volume",
                entry.volume_mut(),
            );

            // コメント外してファイルパスを表示
            /* ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
                if let Some(path) = entry.path().to_str() {
                    ui.weak(path);
                }
            }); */
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
                    action.add_undo();
                }
            });
        }
    }
}
