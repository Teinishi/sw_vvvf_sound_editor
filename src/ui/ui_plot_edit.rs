use crate::{
    app::AppAction,
    editable_function::EditableFunction,
    state::{AudioEntry, AudioEntryId, Cursor},
    ui::PlotAutoColor,
};
use egui::{Id, Modifiers, Pos2, Rangef, Response, Stroke};
use egui_plot::{
    GridInput, GridMark, Line, Plot, PlotPoint, PlotPoints, PlotTransform, Points, Polygon, VLine,
    log_grid_spacer,
};

#[derive(Debug)]
pub struct PlotEditEntry<'a, T> {
    pub func: &'a mut EditableFunction,
    pub color: egui::Color32,
    pub name: String,
    pub id: T,
}

impl<'a, T> PlotEditEntry<'a, T> {
    pub fn new(func: &'a mut EditableFunction, color: egui::Color32, name: String, id: T) -> Self {
        Self {
            func,
            color,
            name,
            id,
        }
    }
}

impl<'a> PlotEditEntry<'a, AudioEntryId> {
    pub fn pitch(audio_entries: &'a mut [AudioEntry]) -> Vec<Self> {
        audio_entries
            .iter_mut()
            .enumerate()
            .map(|(i, e)| {
                let id = e.path().clone();
                Self {
                    func: e.pitch_mut(),
                    color: PlotAutoColor::get_color(i),
                    name: format!("Pitch {i}"),
                    id,
                }
            })
            .collect()
    }

    pub fn volume(audio_entries: &'a mut [AudioEntry]) -> Vec<Self> {
        audio_entries
            .iter_mut()
            .enumerate()
            .map(|(i, e)| {
                let id = e.path().clone();
                Self {
                    func: e.volume_mut(),
                    color: PlotAutoColor::get_color(i),
                    name: format!("Volume {i}"),
                    id,
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

    fn drag_point(
        &self,
        func: &mut EditableFunction,
        modifiers: &Modifiers,
        transform: &PlotTransform,
        pointer_plot_pos: PlotPoint,
        grid_data: &PlotGridData,
    ) -> bool {
        if matches!(
            func.mode,
            crate::editable_function::EditableFunctionMode::Points
        ) {
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
        cursor: &mut Option<&mut Cursor>,
        init_plot: impl FnOnce() -> Plot<'static>,
        inside_plot: impl FnOnce(&mut egui_plot::PlotUi<'_>),
    ) where
        T: PartialEq + Clone,
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

        let mut clicked = plot_response.response.clicked();
        for PlotEditEntry {
            func,
            color: _,
            name,
            id,
        } in entries.iter_mut()
        {
            let is_selected = selection.as_ref() == Some(id);
            let pointer_coordinate = plot_response.inner;

            // 点のドラッグ移動を反映
            if let (Some(dragging_point), Some(pointer)) =
                (self.dragging_point.as_ref(), pointer_coordinate)
            {
                if selection.as_ref() == Some(&dragging_point.id) {
                    let changed = dragging_point.drag_point(
                        func,
                        &ui.ctx().input(|i| i.modifiers),
                        &plot_response.transform,
                        pointer,
                        &grid_data,
                    );
                    if changed {
                        action.add_undo();
                    }
                }
            }

            // 点の削除
            if let Some((id, index)) = remove_point.as_ref() {
                if selection.as_ref() == Some(id) {
                    func.remove_point(*index);
                    action.add_undo();
                }
            }

            // 線のクリック処理
            if clicked && plot_response.hovered_plot_item == Some(Id::new(name)) {
                if is_selected {
                    if let Some(pointer) = pointer_coordinate {
                        func.split_segment(pointer.x);
                        action.add_undo();
                        clicked = false;
                    }
                } else {
                    *selection = Some(id.clone());
                    clicked = false;
                }
            }
        }

        // 何もないところをクリックしたとき
        if clicked {
            *selection = None;
            if let (Some(cursor), Some(pointer)) = (cursor.as_deref_mut(), plot_response.inner) {
                cursor.set_spot(pointer.x);
            }
        }
    }

    fn plot_content<'a, 'b>(
        &mut self,
        plot_ui: &mut egui_plot::PlotUi<'a>,
        action: &mut AppAction,
        entries: &'b [PlotEditEntry<'b, T>],
        selection: &Option<T>,
        mouse_down: bool,
    ) -> Option<(T, usize)>
    where
        'b: 'a,
        T: PartialEq + Clone,
    {
        let marker_radius: f32 = 8.0;
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
        } in entries
        {
            let is_selected = selection.as_ref() == Some(id);

            let marker = if is_selected
                && matches!(
                    func.mode,
                    crate::editable_function::EditableFunctionMode::Points
                ) {
                // マーカークリック・ドラッグ
                for (j, p) in func.points().iter().enumerate() {
                    let screen_pos = plot_ui.screen_from_plot(PlotPoint::new(p.0, p.1));
                    if let (Some(pointer_screen_pos), Some(pointer_plot_pos)) =
                        (pointer_screen_pos, plot_ui.pointer_coordinate())
                    {
                        if pointer_screen_pos.distance_sq(screen_pos) < marker_radius.powi(2) {
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
                let points: Vec<[f64; 2]> = func.points().iter().map(|p| [p.0, p.1]).collect();
                let marker = Points::new(format!("{name} marker"), points)
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
                name.clone(),
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

        remove_point
    }

    fn show_cursor(plot_ui: &mut egui_plot::PlotUi<'_>, cursor: &Cursor) {
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

    /*fn remove_point(&mut self, index: usize, func: &mut EditableFunction) {
        if self.dragging_point.is_some_and(|p| i == index) {
            self.dragging_point = None;
        }
        func.remove_point(index);
    }*/

    fn plot_drag(
        plot_ui: &mut egui_plot::PlotUi<'_>,
        cursor: &mut Option<&mut Cursor>,
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
    fn apply<'a>(&'a self, plot: Plot<'a>) -> Plot<'a> {
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
