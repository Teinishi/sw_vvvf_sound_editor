use egui::Id;
use std::path::PathBuf;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct UiAudioFiles {
    inactive_audio_files: Vec<AudioFileEntry>,
    active_audio_files: Vec<AudioFileEntry>,
}

impl UiAudioFiles {
    pub fn clear(&mut self) {
        self.inactive_audio_files.clear();
        self.active_audio_files.clear();
    }

    pub fn add_inactive_audio_file(&mut self, path: PathBuf) {
        self.inactive_audio_files.push(AudioFileEntry { path });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut dnd_from_to: Option<(AudioDndLocation, AudioDndLocation)> = None;

        ui.columns(2, |uis| {
            for (col_idx, col) in [AudioDndColumn::Inactive, AudioDndColumn::Active]
                .into_iter()
                .enumerate()
            {
                let ui = &mut uis[col_idx];
                let entries = col.entries(self);

                ui.label(col.name());

                let frame = egui::Frame::default().inner_margin(4.0);

                let (_, dropped_payload) = ui.dnd_drop_zone::<AudioDndLocation, ()>(frame, |ui| {
                    ui.set_min_size(egui::vec2(64.0, 100.0));

                    for (row, entry) in entries.iter().enumerate() {
                        let location = AudioDndLocation { col, row };

                        let id = Id::new(("ui_audio_files_entry", entry.path.clone()));
                        let response = ui
                            .dnd_drag_source(id, location, |ui| {
                                entry.ui_dnd(ui);
                            })
                            .response;

                        if let (Some(pointer), Some(hovered_payload)) = (
                            ui.input(|i| i.pointer.interact_pos()),
                            response.dnd_hover_payload::<AudioDndLocation>(),
                        ) {
                            let rect = response.rect;

                            // Preview insertion:
                            let stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
                            let insert_row_idx = if *hovered_payload == location {
                                // We are dragged onto ourselves
                                ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                                row
                            } else if pointer.y < rect.center().y {
                                // Above us
                                ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                row
                            } else {
                                // Below us
                                ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                row + 1
                            };

                            if let Some(dragged_payload) = response.dnd_release_payload() {
                                // The user dropped onto this item.
                                dnd_from_to = Some((
                                    *dragged_payload,
                                    AudioDndLocation {
                                        col,
                                        row: insert_row_idx,
                                    },
                                ));
                            }
                        }
                    }
                });

                if let Some(dropped_payload) = dropped_payload {
                    // The user dropped onto the column, but not on any one item.
                    dnd_from_to = Some((
                        *dropped_payload,
                        AudioDndLocation {
                            col,
                            row: usize::MAX, // Inset last
                        },
                    ));
                }
            }
        });

        if let Some((from, mut to)) = dnd_from_to {
            if from.col == to.col && from.row < to.row {
                to.row -= 1;
            }
            let item = from.col.entries_mut(self).remove(from.row);
            let to_enteis = to.col.entries_mut(self);
            to_enteis.insert(to.row.min(to_enteis.len()), item);
        }
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct AudioFileEntry {
    path: PathBuf,
}

impl AudioFileEntry {
    fn name(&self) -> Option<&str> {
        self.path.file_name().and_then(|n| n.to_str())
    }

    fn ui_dnd(&self, ui: &mut egui::Ui) {
        if let Some(name) = self.name() {
            ui.label(name);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AudioDndColumn {
    Inactive,
    Active,
}

impl AudioDndColumn {
    fn name(&self) -> &'static str {
        match self {
            Self::Inactive => "Inactive Files",
            Self::Active => "Active Files",
        }
    }

    fn entries<'a>(&self, ui_audio_files: &'a UiAudioFiles) -> &'a Vec<AudioFileEntry> {
        match self {
            Self::Inactive => &ui_audio_files.inactive_audio_files,
            Self::Active => &ui_audio_files.active_audio_files,
        }
    }

    fn entries_mut<'a>(&self, ui_audio_files: &'a mut UiAudioFiles) -> &'a mut Vec<AudioFileEntry> {
        match self {
            Self::Inactive => &mut ui_audio_files.inactive_audio_files,
            Self::Active => &mut ui_audio_files.active_audio_files,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AudioDndLocation {
    col: AudioDndColumn,
    row: usize,
}
