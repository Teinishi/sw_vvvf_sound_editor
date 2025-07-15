use crate::state::{AudioEntryId, EditableFunction};
use egui::{Id, Pos2, Response};
use egui_plot::{GridMark, Line, Plot, PlotPoint, PlotPoints, Points};

#[derive(Debug, Default)]
pub struct UiFunctionEdit {
    default_bounds_modified: bool,
    dragging_point: Option<(usize, (f64, f64), Pos2)>,
    last_pointer_button_down: bool,
}

impl UiFunctionEdit {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        entries: &mut [(AudioEntryId, egui::Color32, &mut EditableFunction)],
        selection: &mut Option<AudioEntryId>,
        init_plot: impl FnOnce() -> Plot<'static>,
        inside_plot: impl FnOnce(&mut egui_plot::PlotUi<'_>),
    ) {
        let plot = init_plot()
            .allow_drag(false)
            .allow_double_click_reset(false);

        if selection.is_none() {
            self.dragging_point = None;
        }
        let plot_response = plot.show(ui, |plot_ui| {
            let r = self.plot_content(plot_ui, entries, selection);
            inside_plot(plot_ui);
            r
        });

        // 線のクリック処理
        let mut clicked = plot_response.response.clicked();
        for (index, (path, _, func)) in entries.iter_mut().enumerate() {
            if plot_response.hovered_plot_item != Some(Id::new(format!("Volume {index}"))) {
                continue;
            }
            let is_selected = selection.as_ref() == Some(path);

            let pointer_coordinate = plot_response.inner;
            if clicked && is_selected {
                if let Some(pointer) = pointer_coordinate {
                    func.split_segment(pointer.x);
                    clicked = false;
                }
            }

            if clicked {
                *selection = Some(path.clone());
                clicked = false;
            }
        }
        if clicked {
            *selection = None;
        }
    }

    fn plot_content<'a>(
        &mut self,
        plot_ui: &mut egui_plot::PlotUi<'a>,
        entries: &'a mut [(AudioEntryId, egui::Color32, &mut EditableFunction)],
        selection: &Option<AudioEntryId>,
    ) -> Option<PlotPoint> {
        let marker_radius: f32 = 8.0;
        let response = plot_ui.response();
        let width_usize = response.rect.width().round() as usize;

        let mouse_down = self.check_mouse_down(response);
        if !response.is_pointer_button_down_on() || selection.is_none() {
            self.dragging_point = None;
        }

        let clicked_secondary = response.clicked_by(egui::PointerButton::Secondary);
        let pointer_screen_pos = response.interact_pointer_pos();

        let mut remove_point_index = None;
        for (index, (path, color, func)) in entries.iter_mut().enumerate() {
            let is_selected = selection.as_ref() == Some(path);

            let marker = if is_selected {
                // マーカークリック・ドラッグ
                for (j, p) in func.points().iter().enumerate() {
                    let screen_pos = plot_ui.screen_from_plot(PlotPoint::new(p.0, p.1));
                    if let Some(pointer_screen_pos) = pointer_screen_pos {
                        if pointer_screen_pos.distance_sq(screen_pos) < marker_radius.powi(2) {
                            if mouse_down {
                                self.dragging_point = Some((j, *p, pointer_screen_pos));
                            }
                            if clicked_secondary {
                                remove_point_index = Some(j);
                            }
                        }
                    }
                }

                // ドラッグ移動反映
                self.point_drag(plot_ui, func);

                // 点を削除
                if let Some(index) = remove_point_index {
                    self.remove_point(index, func);
                }

                // マーカー描画
                let points: Vec<[f64; 2]> = func.points().iter().map(|p| [p.0, p.1]).collect();
                let marker = Points::new(format!("Volume {index} marker"), points)
                    .highlight(true)
                    .radius(marker_radius / 2f32.sqrt())
                    .shape(egui_plot::MarkerShape::Diamond)
                    .color(*color)
                    .filled(false)
                    .allow_hover(false);
                Some(marker)
            } else {
                None
            };

            // 線描画
            let line = Line::new(
                format!("Volume {index}"),
                PlotPoints::from_explicit_callback(move |x| func.value_at(x), .., width_usize),
            )
            .width(2.0)
            .color(*color)
            .highlight(is_selected);
            plot_ui.line(line);

            // マーカーは線より上に表示
            if let Some(marker) = marker {
                plot_ui.points(marker);
            }
        }

        // パン制御
        self.plot_drag(plot_ui, self.dragging_point.is_none());
        // デフォルトに戻す
        //plot_ui.set_plot_bounds(self.default_bounds);

        plot_ui.pointer_coordinate()
    }

    fn check_mouse_down(&mut self, response: &Response) -> bool {
        let down = response.is_pointer_button_down_on();
        let mouse_down_now = down && !self.last_pointer_button_down;
        self.last_pointer_button_down = down;
        mouse_down_now
    }

    fn point_drag(&self, plot_ui: &egui_plot::PlotUi<'_>, func: &mut EditableFunction) {
        // 点のドラッグ処理
        if let (Some((index, start_val, pointer_start_pos)), Some(pointer_current_pos)) = (
            self.dragging_point,
            plot_ui.response().interact_pointer_pos(),
        ) {
            let mut val = plot_ui.plot_from_screen(pointer_current_pos);
            let shift = plot_ui.ctx().input(|input| input.modifiers.shift_only());
            if shift {
                // 縦横スナップ
                if (pointer_current_pos.x - pointer_start_pos.x).abs()
                    < (pointer_current_pos.y - pointer_start_pos.y).abs()
                {
                    val.x = start_val.0;
                } else {
                    val.y = start_val.1;
                }
            }
            func.move_point_to(index, (val.x, val.y));
        }
    }

    fn remove_point(&mut self, index: usize, func: &mut EditableFunction) {
        if self.dragging_point.is_some_and(|(i, _, _)| i == index) {
            self.dragging_point = None;
        }
        func.remove_point(index);
    }

    fn plot_drag(&mut self, plot_ui: &mut egui_plot::PlotUi<'_>, enable: bool) {
        if enable {
            // ドラッグで移動しつつ、第一象限から外れないように
            let delta = plot_ui.pointer_coordinate_drag_delta();
            if delta.length_sq() > 1e-6 {
                self.default_bounds_modified = true;
            }
            let mut bounds = plot_ui.plot_bounds();
            bounds.translate((-delta.x as f64, -delta.y as f64));
            let bounds_min = bounds.min();
            bounds.translate(((-bounds_min[0]).max(0.0), (-bounds_min[1]).max(0.0)));
            plot_ui.set_plot_bounds(bounds);
        } else {
            // なぜかこれをしないとデフォルト位置に戻され
            plot_ui.set_plot_bounds(plot_ui.plot_bounds());
        }
    }
}

pub fn aixs_hint_formatter_percentage(mark: GridMark, _: &std::ops::RangeInclusive<f64>) -> String {
    let percent = 100.0 * mark.value;
    if is_approx_integer(percent) && percent < 100.5 {
        format!("{percent:.0}%")
    } else {
        String::new()
    }
}

fn is_approx_integer(val: f64) -> bool {
    val.fract().abs() < 1e-6
}
