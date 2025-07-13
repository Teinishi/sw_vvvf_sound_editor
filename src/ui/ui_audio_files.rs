#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::state::{AudioEntry, State};
use egui::Button;
use egui::{Id, Label, ScrollArea};
use egui_extras::{Size, StripBuilder};

#[derive(Debug, Default)]
pub struct UiAudioFiles;

impl UiAudioFiles {
    #[expect(clippy::unused_self)]
    pub fn ui(&self, ui: &mut egui::Ui, frame: Option<&eframe::Frame>, state: &mut State) {
        let mut dnd_from_to: Option<(DndLocation, DndLocation)> = None;

        StripBuilder::new(ui)
            .size(Size::exact(20.0))
            .size(Size::remainder())
            .sizes(Size::exact(20.0), 2)
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    ui.label("Audio Files");
                });

                strip.cell(|ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui_list_dnd(ui, &state.audio_entries, &mut dnd_from_to);
                    });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::relative(0.3))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                let (_, dropped_payload) = ui.dnd_drop_zone::<DndLocation, ()>(
                                    egui::Frame::default(),
                                    |ui| {
                                        ui.set_min_size(ui.available_size());
                                        ui.centered_and_justified(|ui| {
                                            ui.label("Discard");
                                        });
                                    },
                                );
                                if let Some(dropped_payload) = dropped_payload {
                                    // The user dropped onto the column
                                    dnd_from_to = Some((*dropped_payload, DndLocation::Discard));
                                }
                            });

                            strip.cell(|ui| {
                                if ui
                                    .add_sized(ui.available_size(), Button::new("All"))
                                    .clicked()
                                {
                                    state.clear_entries();
                                }
                            });
                        });
                });

                strip.cell(|ui| {
                    if ui
                        .add_sized(ui.available_size(), Button::new("Add"))
                        .clicked()
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(paths) = add_audio_file_dialog(frame) {
                            for path in paths {
                                state.add_audio_entry(path);
                            }
                        }
                    }
                });
            });

        if let Some((DndLocation::Active(from_idx), to)) = dnd_from_to {
            match to {
                DndLocation::Active(mut to_idx) => {
                    if from_idx < to_idx {
                        to_idx -= 1;
                    }
                    state.move_entry(from_idx, to_idx);
                }
                DndLocation::Discard => {
                    state.remove_entry(from_idx);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DndLocation {
    Active(usize),
    Discard,
}

fn ui_list_dnd(
    ui: &mut egui::Ui,
    entries: &[AudioEntry],
    dnd_from_to: &mut Option<(DndLocation, DndLocation)>,
) {
    let (_, dropped_payload) =
        ui.dnd_drop_zone::<DndLocation, ()>(egui::Frame::default().inner_margin(4.0), |ui| {
            ui.set_min_size(ui.available_size());

            for (row_idx, entry) in entries.iter().enumerate() {
                let location = DndLocation::Active(row_idx);

                let id = Id::new(("ui_audio_files_entry", entry.path().clone()));
                let response = ui
                    .dnd_drag_source(id, location, |ui| {
                        ui.set_width(ui.available_width());
                        if let Some(name) = entry.name() {
                            ui_entry_dnd(ui, Some(row_idx), &name);
                        }
                    })
                    .response;

                if let (Some(pointer), Some(hovered_payload)) = (
                    ui.input(|i| i.pointer.interact_pos()),
                    response.dnd_hover_payload::<DndLocation>(),
                ) {
                    let rect = response.rect;

                    // Preview insertion:
                    let stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
                    let insert_row_idx = if *hovered_payload == location {
                        // We are dragged onto ourselves
                        ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                        row_idx
                    } else if pointer.y < rect.center().y {
                        // Above us
                        ui.painter().hline(rect.x_range(), rect.top(), stroke);
                        row_idx
                    } else {
                        // Below us
                        ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                        row_idx + 1
                    };

                    if let Some(dragged_payload) = response.dnd_release_payload() {
                        // The user dropped onto this item.
                        *dnd_from_to =
                            Some((*dragged_payload, DndLocation::Active(insert_row_idx)));
                    }
                }
            }
        });

    if let Some(dropped_payload) = dropped_payload {
        // The user dropped onto the column, but not on any one item.
        *dnd_from_to = Some((
            *dropped_payload,
            DndLocation::Active(usize::MAX), // Inset last
        ));
    }
}

fn ui_entry_dnd(ui: &mut egui::Ui, index: Option<usize>, name: &str) {
    ui.horizontal(|ui| {
        if let Some(index) = index {
            ui.strong(format!("{index}."));
        }
        ui.add(Label::new(name).truncate());
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn add_audio_file_dialog<
    W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
>(
    parent: Option<&W>,
) -> Option<Vec<PathBuf>> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(p) = parent {
        dialog = dialog.set_parent(p);
    }
    dialog = dialog.add_filter("Ogg Vorbis File", &["ogg"]);
    dialog.pick_files()
}
