use crate::{player_state::PlayerState, preference::Preference};
use egui::{
    Color32, FontId, Frame, Label, Layout, RichText, Sense, Slider, TextFormat, UiBuilder,
    text::LayoutJob,
};
use egui_extras::{Size, StripBuilder};

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
    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        preference: &mut Preference,
        player_state: &mut PlayerState,
    ) {
        StripBuilder::new(ui)
            .size(Size::exact(100.0))
            .size(Size::exact(160.0))
            .size(Size::remainder())
            .size(Size::exact(200.0))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    // マスコン表示
                    let response = ui
                        .scope_builder(UiBuilder::new().sense(Sense::all()), |ui| {
                            Frame::new().inner_margin(0.0).show(ui, |ui| {
                                Self::ui_master_controller(ui, player_state.master_controller);
                            });
                        })
                        .response;

                    // ホイールでマスコン操作
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
                });

                strip.cell(|ui| {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        Self::ui_speed(ui, player_state.speed);
                    });
                });

                strip.cell(|_| {});

                strip.cell(|ui| {
                    // 音量スライダー
                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            Self::ui_volume(ui, &mut preference.global_volume);
                        },
                    );
                });
            });
    }

    fn ui_master_controller(ui: &mut egui::Ui, m: i32) {
        ui.set_width(120.0);

        ui.centered_and_justified(|ui| {
            let richtext = match m.cmp(&0) {
                std::cmp::Ordering::Greater => {
                    RichText::new(format!("P{m}")).color(color_from_arr(BLUE))
                }
                std::cmp::Ordering::Equal => RichText::new("N").color(color_from_arr(GREEN)),
                std::cmp::Ordering::Less => {
                    RichText::new(format!("B{}", m.saturating_neg())).color(color_from_arr(ORANGE))
                }
            };
            ui.add(Label::new(richtext.size(40.0)).selectable(false));
        });
    }

    fn ui_speed(ui: &mut egui::Ui, speed: f64) {
        let color = ui.ctx().style().visuals.text_color();

        let mut job = LayoutJob::default();

        job.append(
            &format!("{speed:.1}"),
            0.0,
            TextFormat {
                font_id: FontId::proportional(40.0),
                color,
                ..Default::default()
            },
        );

        job.append(
            "km/h",
            10.0,
            TextFormat {
                font_id: FontId::proportional(20.0),
                color,
                ..Default::default()
            },
        );

        ui.label(job);
    }

    fn ui_volume(ui: &mut egui::Ui, volume: &mut f32) {
        ui.horizontal(|ui| {
            ui.label(if *volume <= 0.0 {
                "\u{1f508}"
            } else if *volume < 0.5 {
                "\u{1f509}"
            } else {
                "\u{1f50A}"
            });

            ui.add(
                Slider::new(volume, 0.0..=1.0)
                    .suffix("%")
                    .step_by(0.01)
                    .custom_formatter(|v, _| format!("{:.0}", v * 100.0))
                    .custom_parser(|s| s.parse().ok().map(|v: f64| v / 100.0))
                    .trailing_fill(true),
            );
        });
    }
}
