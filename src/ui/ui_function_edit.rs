use crate::state::{AudioEntry, State};
use egui::Id;
use egui_plot::{AxisHints, Line, Plot, PlotPoint, PlotPoints, Points};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct UiFunctionEdit {
    selected_entry: Option<PathBuf>,
    dragging_point: Option<usize>,
    last_pointer_button_down: bool,
}

impl UiFunctionEdit {
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        let plot = Self::init_plot();

        let plot_response = plot.show(ui, |plot_ui| {
            let marker_radius: f32 = 8.0;
            let response = plot_ui.response();
            let width_usize = response.rect.width().round() as usize;

            let pointer_button_down = response.is_pointer_button_down_on();
            let mouse_down = pointer_button_down && !self.last_pointer_button_down;
            self.last_pointer_button_down = pointer_button_down;
            if !pointer_button_down || self.selected_entry.is_none() {
                self.dragging_point = None;
            }

            let clicked_secondary = response.clicked_by(egui::PointerButton::Secondary);
            let pointer_screen_pos = response.interact_pointer_pos();

            let mut remove_point_index = None;
            let mut color_index = 0;
            for (index, entry) in state.audio_entries.iter_mut().enumerate() {
                let is_selected = self.is_selected(entry);
                let color = Self::auto_color(&mut color_index);

                let marker = if is_selected {
                    // マーカークリック・ドラッグ
                    for (j, p) in entry.volume().points().iter().enumerate() {
                        let screen_pos = plot_ui.screen_from_plot(PlotPoint::new(p.0, p.1));
                        if pointer_screen_pos
                            .is_some_and(|p| p.distance_sq(screen_pos) < (marker_radius).powi(2))
                        {
                            if mouse_down {
                                self.dragging_point = Some(j);
                            }
                            if clicked_secondary {
                                remove_point_index = Some(j);
                            }
                        }
                    }

                    // ドラッグ移動反映
                    if let Some(index) = self.dragging_point {
                        let delta = plot_ui.pointer_coordinate_drag_delta();
                        entry
                            .volume_mut()
                            .move_point(index, (delta.x as f64, delta.y as f64));
                    }

                    // 点を削除
                    if let Some(index) = remove_point_index {
                        if self.dragging_point == Some(index) {
                            self.dragging_point = None;
                        }
                        entry.volume_mut().remove_point(index);
                    }

                    // マーカー描画
                    let points: Vec<[f64; 2]> =
                        entry.volume().points().iter().map(|p| [p.0, p.1]).collect();
                    let marker = Points::new(format!("Volume {index} marker"), points)
                        .highlight(true)
                        .radius(marker_radius / 2f32.sqrt())
                        .shape(egui_plot::MarkerShape::Diamond)
                        .color(color)
                        .filled(false)
                        .allow_hover(false);
                    Some(marker)
                } else {
                    None
                };

                // 線描画
                let line = Line::new(
                    format!("Volume {index}"),
                    PlotPoints::from_explicit_callback(
                        move |x| entry.volume_at(x),
                        ..,
                        width_usize,
                    ),
                )
                .width(2.0)
                .color(color)
                .highlight(is_selected);
                plot_ui.line(line);

                // マーカーは線より上に表示
                if let Some(marker) = marker {
                    plot_ui.points(marker);
                }
            }

            // パン制御
            Self::plot_drag(plot_ui, self.dragging_point.is_none());

            plot_ui.pointer_coordinate()
        });

        // 線のクリック処理
        let mut clicked = plot_response.response.clicked();
        for (index, entry) in state.audio_entries.iter_mut().enumerate() {
            if plot_response.hovered_plot_item != Some(Id::new(format!("Volume {index}"))) {
                continue;
            }
            let is_selected = self.is_selected(entry);

            let pointer_coordinate = plot_response.inner;
            if clicked && is_selected {
                if let Some(pointer) = pointer_coordinate {
                    entry.volume_mut().split_segment(pointer.x);
                    clicked = false;
                }
            }

            if clicked {
                self.selected_entry = Some(entry.path().clone());
                clicked = false;
            }
        }
        if clicked {
            self.selected_entry = None;
        }
    }

    fn init_plot<'a>() -> Plot<'a> {
        Plot::new("volume_graph")
            .show_axes(true)
            .show_grid(true)
            .allow_drag(false)
            .default_x_bounds(0.0, 80.0)
            .default_y_bounds(0.0, 1.1)
            .custom_x_axes(vec![])
            .custom_y_axes(vec![AxisHints::new_y().label("Volume").formatter(
                |mark, _| {
                    let percent = 100.0 * mark.value;
                    if is_approx_integer(percent) && percent < 100.5 {
                        format!("{percent:.0}%")
                    } else {
                        String::new()
                    }
                },
            )])
    }

    fn plot_drag(plot_ui: &mut egui_plot::PlotUi<'_>, enable: bool) {
        if enable {
            // ドラッグで移動しつつ、第一象限から外れないように
            let delta = plot_ui.pointer_coordinate_drag_delta();
            let mut bounds = plot_ui.plot_bounds();
            bounds.translate((-delta.x as f64, -delta.y as f64));
            let bounds_min = bounds.min();
            bounds.translate(((-bounds_min[0]).max(0.0), (-bounds_min[1]).max(0.0)));
            plot_ui.set_plot_bounds(bounds);
        } else {
            // なぜかこれをしないとデフォルト位置に戻される
            plot_ui.translate_bounds(egui::Vec2::ZERO);
        }
    }

    fn auto_color(color_index: &mut i32) -> egui::Color32 {
        let i = *color_index;
        *color_index += 1;
        let golden_ratio = (5f32.sqrt() - 1.0) / 2.0;
        let h = i as f32 * golden_ratio;
        egui::epaint::Hsva::new(h, 0.85, 0.5, 1.0).into()
    }

    fn is_selected(&self, entry: &AudioEntry) -> bool {
        self.selected_entry
            .as_ref()
            .is_some_and(|s| s == entry.path())
    }
}

fn is_approx_integer(val: f64) -> bool {
    val.fract().abs() < 1e-6
}
