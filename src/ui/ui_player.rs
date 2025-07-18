use crate::player_state::PlayerState;
use egui::{Color32, Frame, Label, RichText, Sense, UiBuilder};

//const RED: [u8; 3] = [188, 77, 77];
const ORANGE: [u8; 3] = [188, 138, 77];
const GREEN: [u8; 3] = [154, 188, 77];
const BLUE: [u8; 3] = [77, 123, 188];

fn color_from_arr(arr: [u8; 3]) -> Color32 {
    Color32::from_rgb(arr[0], arr[1], arr[2])
}

#[derive(Debug, Default)]
pub struct UiPlayer;

impl UiPlayer {
    #[expect(clippy::unused_self)]
    pub fn ui(&self, ui: &mut egui::Ui, player_state: &mut PlayerState) {
        let response = ui
            .scope_builder(UiBuilder::new().sense(Sense::all()), |ui| {
                Frame::new().inner_margin(0.0).show(ui, |ui| {
                    ui.set_width(120.0);

                    ui.centered_and_justified(|ui| {
                        let m = player_state.master_controller;
                        let richtext = match m.cmp(&0) {
                            std::cmp::Ordering::Greater => {
                                RichText::new(format!("P{m}")).color(color_from_arr(BLUE))
                            }
                            std::cmp::Ordering::Equal => {
                                RichText::new("N").color(color_from_arr(GREEN))
                            }
                            std::cmp::Ordering::Less => {
                                RichText::new(format!("B{}", m.saturating_neg()))
                                    .color(color_from_arr(ORANGE))
                            }
                        };
                        ui.add(Label::new(richtext.size(40.0)).selectable(false));
                    });
                });
            })
            .response;

        if response.hovered() {
            let wheel = ui.input(|i| i.raw_scroll_delta).y;
            if wheel.abs() > 1e-6 {
                if wheel > 0.0 {
                    player_state.master_controller -= 1;
                } else {
                    player_state.master_controller += 1;
                }
            }
        }
    }
}
