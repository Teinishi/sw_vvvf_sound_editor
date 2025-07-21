use crate::{
    app::AppAction,
    func_edit::FuncEdit,
    state::{AudioEntry, AudioEntryId, SelectionCursor},
    ui::PlotAutoColor,
};
use egui::{Color32, Id, Modifiers, Pos2, Rangef, Response, Stroke};
use egui_plot::{
    GridInput, GridMark, Line, Plot, PlotPoint, PlotPoints, PlotTransform, Points, Polygon, VLine,
    log_grid_spacer,
};
use std::sync::Arc;

const MARKER_RADIUS: f32 = 8.0;

pub struct PlotEditEntry<'a, T> {
    pub func: &'a mut dyn FuncEdit,
    pub color: egui::Color32,
    pub name: String,
    pub id: T,
    pub gradient_color: Option<Arc<dyn Fn(PlotPoint) -> Color32 + Send + Sync>>,
}

impl<'a, T> PlotEditEntry<'a, T> {
    pub fn new<F: FuncEdit>(func: &'a mut F, color: egui::Color32, name: String, id: T) -> Self {
        Self {
            func,
            color,
            name,
            id,
            gradient_color: None,
        }
    }
}

impl<'a> PlotEditEntry<'a, AudioEntryId> {
    pub fn pitch(
        audio_entries: &'a mut [AudioEntry],
        selection: &Option<AudioEntryId>,
    ) -> Vec<Self> {
        audio_entries
            .iter_mut()
            .enumerate()
            .map(|(i, e)| {
                let id = *e.id();
                let is_selected = selection.as_ref() == Some(&id);
                let color = PlotAutoColor::get_color(i);
                let volume_fn = e.volume().clone();
                let gradient_color = move |p: PlotPoint| {
                    // 音量依存で濃さを変更
                    let v = volume_fn.value_at(p.x) as f32;
                    let f = 0.1 + 0.9 * v;
                    color.linear_multiply(f)
                };
                Self {
                    func: e.pitch_mut(),
                    color,
                    name: format!("Pitch {i}"),
                    id,
                    gradient_color: (!is_selected).then_some(Arc::new(gradient_color)),
                }
            })
            .collect()
    }

    pub fn volume(audio_entries: &'a mut [AudioEntry]) -> Vec<Self> {
        audio_entries
            .iter_mut()
            .enumerate()
            .map(|(i, e)| {
                let id = *e.id();
                Self {
                    func: e.volume_mut(),
                    color: PlotAutoColor::get_color(i),
                    name: format!("Volume {i}"),
                    id,
                    gradient_color: None,
                }
            })
            .collect()
    }
}

#[derive(Debug)]
struct DraggingPoint<T> {
    id: T,
    index: usize,
    start_pointer_plot_pos: PlotPoint,
    grab_offset_screen: egui::Vec2,
}

impl<T> DraggingPoint<T> {
    fn new(
        id: T,
        index: usize,
        pointer_plot_pos: PlotPoint,
        point_screen_pos: Pos2,
        pointer_screen_pos: Pos2,
    ) -> Self {
        Self {
            id,
            index,
            start_pointer_plot_pos: pointer_plot_pos,
            grab_offset_screen: pointer_screen_pos - point_screen_pos,
        }
    }

    fn drag_point<F: ?Sized + FuncEdit>(
        &self,
        func: &mut F,
        modifiers: &Modifiers,
        transform: &PlotTransform,
        pointer_plot_pos: PlotPoint,
        grid_data: &PlotGridData,
    ) -> bool {
        if func.is_mode_points() {
            func.move_point_to(
                self.index,
                self.get_drag_point_pos(modifiers, transform, pointer_plot_pos, grid_data),
            );
            true
        } else {
            false
        }
    }

    fn get_drag_point_pos(
        &self,
        modifiers: &Modifiers,
        transform: &PlotTransform,
        pointer_plot_pos: PlotPoint,
        grid_data: &PlotGridData,
    ) -> (f64, f64) {
        // 点のドラッグ処理
        let mut pointer_plot_pos = (pointer_plot_pos.x, pointer_plot_pos.y);
        if modifiers.shift_only() {
            // Shiftキーで縦横スナップ
            if (pointer_plot_pos.0 - self.start_pointer_plot_pos.x).abs()
                < (pointer_plot_pos.1 - self.start_pointer_plot_pos.y).abs()
            {
                pointer_plot_pos.0 = self.start_pointer_plot_pos.x;
            } else {
                pointer_plot_pos.0 = self.start_pointer_plot_pos.y;
            }
        } else if modifiers.command_only() {
            // Ctrlキーでグリッドにスナップ
            if let Some(snap_to) = grid_data.nearest_point(transform, &pointer_plot_pos) {
                return snap_to;
            }
        }
        let x = pointer_plot_pos.0 - self.grab_offset_screen.x as f64 / transform.dpos_dvalue_x();
        let y = pointer_plot_pos.1 - self.grab_offset_screen.y as f64 / transform.dpos_dvalue_y();
        (x, y)
    }
}

#[derive(Debug)]
pub struct UiPlotEdit<T> {
    dragging_point: Option<DraggingPoint<T>>,
    last_pointer_button_down: bool,
}

impl<T> Default for UiPlotEdit<T> {
    fn default() -> Self {
        Self {
            dragging_point: None,
            last_pointer_button_down: false,
        }
    }
}

impl<T> UiPlotEdit<T> {
    #[expect(clippy::too_many_arguments)]
    pub fn ui<'a>(
        &mut self,
        ui: &mut egui::Ui,
        action: &mut AppAction,
        entries: &'a mut [PlotEditEntry<'a, T>],
        selection: &mut Option<T>,
        cursor: &mut Option<&mut SelectionCursor>,
        init_plot: impl FnOnce() -> Plot<'static>,
        inside_plot: impl FnOnce(&mut egui_plot::PlotUi<'_>),
    ) where
        T: PartialEq + Clone + std::fmt::Debug,
    {
        let grid_data = PlotGridData::default();
        let plot = grid_data.apply(
            init_plot()
                .allow_drag(false)
                .allow_double_click_reset(false),
        );

        if selection.is_none() {
            self.dragging_point = None;
        }
        let shift_only = ui.input(|i| i.modifiers.shift_only());

        let mut remove_point = None;

        let plot_response = plot.show(ui, |plot_ui| {
            let mouse_down = self.check_mouse_down(plot_ui.response());

            remove_point = self.plot_content(plot_ui, action, entries, selection, mouse_down);

            if let Some(cursor) = cursor.as_deref() {
                Self::show_cursor(plot_ui, cursor);
            }
            Self::plot_drag(
                plot_ui,
                cursor,
                shift_only && self.dragging_point.is_none(),
                mouse_down,
            );

            inside_plot(plot_ui);

            plot_ui.pointer_coordinate()
        });

        let transform = plot_response.transform;
        let hovered_plot_item = plot_response.hovered_plot_item;
        let pointer_coordinate = plot_response.inner;

        // クリック処理
        let mut clicked = plot_response.response.clicked();

        for PlotEditEntry {
            func,
            color: _,
            name: _,
            id,
            gradient_color: _,
        } in entries.iter_mut()
        {
            // 点のドラッグ移動を反映
            if let (Some(dragging_point), Some(pointer)) =
                (self.dragging_point.as_ref(), pointer_coordinate)
            {
                if id == &dragging_point.id {
                    dragging_point.drag_point(
                        *func,
                        &ui.ctx().input(|i| i.modifiers),
                        &transform,
                        pointer,
                        &grid_data,
                    );
                }
            }

            // 点の削除
            if let Some((remove_id, index)) = remove_point.as_ref() {
                if id == remove_id {
                    func.remove_point(*index);
                    action.add_undo();
                }
            }

            // 点の追加
            if clicked && selection.as_ref() == Some(id) {
                if let Some(pointer) = pointer_coordinate {
                    let dpos_dvalue = transform.dpos_dvalue_y();
                    if (pointer.y - func.value_at(pointer.x)).abs()
                        < -(MARKER_RADIUS as f64) / dpos_dvalue
                    {
                        func.split_segment(pointer.x);
                        action.add_undo();
                        clicked = false;
                    }
                }
            }
        }

        // 線をクリックして選択 (点の追加より優先度を下げる)
        if clicked {
            for PlotEditEntry {
                func: _,
                color: _,
                name,
                id,
                gradient_color: _,
            } in entries.iter_mut()
            {
                if Some(Id::new(name)) == hovered_plot_item {
                    *selection = Some(id.clone());
                    clicked = false;
                }
            }
        }

        // 何もないところをクリックしたとき
        if clicked {
            *selection = None;
            if let (Some(cursor), Some(pointer)) = (cursor.as_deref_mut(), pointer_coordinate) {
                cursor.set_spot(pointer.x);
            }
        }
    }

    fn plot_content<'a, 'b>(
        &mut self,
        plot_ui: &mut egui_plot::PlotUi<'a>,
        action: &mut AppAction,
        entries: &'a [PlotEditEntry<'b, T>],
        selection: &Option<T>,
        mouse_down: bool,
    ) -> Option<(T, usize)>
    where
        'b: 'a,
        T: PartialEq + Clone,
    {
        let response = plot_ui.response();
        let width_usize = response.rect.width().round() as usize;

        if self.dragging_point.is_some()
            && (!response.is_pointer_button_down_on() || selection.is_none())
        {
            self.dragging_point = None;
            action.add_undo();
        }

        let clicked_secondary = response.clicked_by(egui::PointerButton::Secondary);
        let pointer_screen_pos = response.interact_pointer_pos();

        let mut remove_point = None;

        for PlotEditEntry {
            func,
            color,
            name,
            id,
            gradient_color,
        } in entries
        {
            let is_selected = selection.as_ref() == Some(id);

            let marker = if is_selected && func.is_mode_points() {
                if let Some(points) = func.points() {
                    // マーカークリック・ドラッグ
                    for (j, p) in points.iter().enumerate() {
                        let screen_pos = plot_ui.screen_from_plot(PlotPoint::new(p.0, p.1));
                        if let (Some(pointer_screen_pos), Some(pointer_plot_pos)) =
                            (pointer_screen_pos, plot_ui.pointer_coordinate())
                        {
                            if pointer_screen_pos.distance_sq(screen_pos) < MARKER_RADIUS.powi(2) {
                                if mouse_down {
                                    self.dragging_point = Some(DraggingPoint::new(
                                        id.clone(),
                                        j,
                                        pointer_plot_pos,
                                        screen_pos,
                                        pointer_screen_pos,
                                    ));
                                }
                                if clicked_secondary {
                                    remove_point = Some((id.clone(), j));
                                }
                            }
                        }
                    }

                    // マーカー描画
                    let points: Vec<[f64; 2]> = points.iter().map(|p| [p.0, p.1]).collect();
                    let marker = Points::new(format!("{name} marker"), points)
                        .highlight(true)
                        .radius(MARKER_RADIUS / 2f32.sqrt())
                        .shape(egui_plot::MarkerShape::Diamond)
                        .color(*color)
                        .filled(false)
                        .allow_hover(false);
                    Some(marker)
                } else {
                    None
                }
            } else {
                None
            };

            // 線描画
            let mut line = Line::new(
                name.clone(),
                PlotPoints::from_explicit_callback(|x| func.value_at(x), .., width_usize),
            )
            .width(2.0)
            .highlight(is_selected);
            if let Some(gradient_color) = gradient_color {
                line = line.gradient_color(gradient_color.clone(), false);
            } else {
                line = line.color(*color);
            }
            plot_ui.line(line);

            // マーカーは線より上に表示
            if let Some(marker) = marker {
                plot_ui.points(marker);
            }
        }

        remove_point
    }

    fn show_cursor(plot_ui: &mut egui_plot::PlotUi<'_>, cursor: &SelectionCursor) {
        let selection_visuals = plot_ui.ctx().style().visuals.selection;

        if let Some((start, end)) = cursor.range() {
            let bounds_y = plot_ui.plot_bounds().range_y();
            let bottom = *bounds_y.start();
            let top = *bounds_y.end();
            let (bottom, top) = (bottom - (top - bottom), top + (top - bottom));
            plot_ui.polygon(
                Polygon::new(
                    "",
                    vec![[start, bottom], [start, top], [end, top], [end, bottom]],
                )
                .fill_color(selection_visuals.bg_fill.linear_multiply(0.2))
                .stroke(Stroke::NONE)
                .allow_hover(false),
            );
        }

        if let Some((a, b)) = cursor.range() {
            plot_ui.vline(
                VLine::new("", a)
                    .width(selection_visuals.stroke.width)
                    .color(selection_visuals.stroke.color)
                    .allow_hover(false),
            );
            plot_ui.vline(
                VLine::new("", b)
                    .width(selection_visuals.stroke.width)
                    .color(selection_visuals.stroke.color)
                    .allow_hover(false),
            );
        }
    }

    fn check_mouse_down(&mut self, response: &Response) -> bool {
        let down = response.is_pointer_button_down_on();
        let mouse_down_now = down && !self.last_pointer_button_down;
        self.last_pointer_button_down = down;
        mouse_down_now
    }

    fn plot_drag(
        plot_ui: &mut egui_plot::PlotUi<'_>,
        cursor: &mut Option<&mut SelectionCursor>,
        pan: bool,
        mouse_down: bool,
    ) -> bool {
        let mut dragged = false;
        let mut bounds = plot_ui.plot_bounds();

        if pan {
            // ドラッグで移動
            let delta = plot_ui.pointer_coordinate_drag_delta();
            if delta.length_sq() > 1e-6 {
                dragged = true;
                bounds.translate((-delta.x as f64, -delta.y as f64));
            }
        } else if let (Some(cursor), Some(pointer)) =
            (cursor.as_deref_mut(), plot_ui.pointer_coordinate())
        {
            if mouse_down {
                // 範囲選択開始
                cursor.set_spot(pointer.x);
            } else if plot_ui.response().is_pointer_button_down_on() {
                cursor.extend(pointer.x);
            }
        }

        //第一象限から外れないように
        let bounds_min = bounds.min();
        bounds.translate(((-bounds_min[0]).max(0.0), (-bounds_min[1]).max(0.0)));
        plot_ui.set_plot_bounds(bounds);

        dragged
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

struct PlotGridData {
    x_spacer: Box<dyn Fn(GridInput) -> Vec<GridMark>>,
    y_spacer: Box<dyn Fn(GridInput) -> Vec<GridMark>>,
    spacing: Rangef,
}

impl Default for PlotGridData {
    fn default() -> Self {
        Self {
            x_spacer: Box::new(log_grid_spacer(10)),
            y_spacer: Box::new(log_grid_spacer(10)),
            spacing: Rangef::new(8.0, 300.0),
        }
    }
}

impl PlotGridData {
    fn apply<'a, 'b>(&'a self, plot: Plot<'b>) -> Plot<'b>
    where
        'a: 'b,
    {
        plot.x_grid_spacer(&self.x_spacer)
            .y_grid_spacer(&self.y_spacer)
            .grid_spacing(self.spacing)
    }

    fn get_grid(&self, transform: &PlotTransform) -> (Vec<GridMark>, Vec<GridMark>) {
        let plot_bounds = transform.bounds();
        let bmin = plot_bounds.min();
        let bmax = plot_bounds.max();
        let grid_x = (self.x_spacer)(GridInput {
            bounds: (bmin[0], bmax[0]),
            base_step_size: self.spacing.min as f64 / transform.dpos_dvalue_x().abs(),
        });
        let grid_y = (self.x_spacer)(GridInput {
            bounds: (bmin[1], bmax[1]),
            base_step_size: self.spacing.min as f64 / transform.dpos_dvalue_y().abs(),
        });
        (grid_x, grid_y)
    }

    fn nearest_point(&self, transform: &PlotTransform, point: &(f64, f64)) -> Option<(f64, f64)> {
        let (grid_x, grid_y) = self.get_grid(transform);

        let cx = grid_x
            .iter()
            .map(|g| (g.value, (g.value - point.0).abs()))
            .reduce(|a, b| if a.1 < b.1 { a } else { b })
            .map(|a| a.0);
        let cy = grid_y
            .iter()
            .map(|g| (g.value, (g.value - point.1).abs()))
            .reduce(|a, b| if a.1 < b.1 { a } else { b })
            .map(|a| a.0);

        cx.zip(cy)
    }
}
