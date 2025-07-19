use super::{PlotAutoColor, UiPlotEdit};
use crate::{
    app::AppAction,
    state::TrainPerformance,
    ui::{UiFunctionEdit, ui_plot_edit::PlotEditEntry},
};
use egui::{Atom, Button, Grid, ScrollArea, Sides, Slider, Window};
use egui_plot::{AxisHints, Plot};

#[derive(Debug, Clone, PartialEq, Eq)]
enum PlotItem {
    Acceleration,
    Drag,
}

#[derive(Debug, Default)]
pub struct UiPerformanceWindow {
    acceleration_plot: UiPlotEdit<PlotItem>,
    selection: Option<PlotItem>,
}

impl UiPerformanceWindow {
    pub const TITLE: &str = "Train Performance";

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        action: &mut AppAction,
        train_performance: &mut TrainPerformance,
    ) {
        Window::new(Self::TITLE)
            .open(open)
            .default_size([325.0, 400.0])
            .min_size([325.0, 300.0])
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.ui_plot(ui, action, train_performance);
                    ui.separator();
                    self.ui_form(ui, action, train_performance);

                    ui.allocate_space(ui.available_size());
                });
            });
    }

    fn ui_plot(
        &mut self,
        ui: &mut egui::Ui,
        action: &mut AppAction,
        train_performance: &mut TrainPerformance,
    ) {
        let mut reset_viewport = false;

        Sides::new().show(
            ui,
            |_| {},
            |ui| {
                reset_viewport = ui.button("Reset viewport").clicked();
            },
        );

        self.acceleration_plot.ui(
            ui,
            action,
            &mut [
                PlotEditEntry::new(
                    &mut train_performance.acceleration,
                    PlotAutoColor::get_color(0),
                    "Acceleration".to_owned(),
                    PlotItem::Acceleration,
                ),
                PlotEditEntry::new(
                    &mut train_performance.drag,
                    PlotAutoColor::get_color(1),
                    "Drag".to_owned(),
                    PlotItem::Drag,
                ),
            ],
            &mut self.selection,
            &mut None,
            || {
                Plot::new("plot_edit_acceleration")
                    .show_axes(true)
                    .show_grid(true)
                    .default_x_bounds(0.0, 150.0)
                    .default_y_bounds(0.0, 4.0)
                    .custom_x_axes(vec![AxisHints::new_x().label("Speed (km/h)")])
                    .custom_y_axes(vec![AxisHints::new_y().label("Acceleration (km/h/s)")])
                    .height(200.0)
            },
            |plot_ui: &mut egui_plot::PlotUi<'_>| {
                if reset_viewport {
                    plot_ui.set_auto_bounds(true);
                }
            },
        );

        ui.columns(2, |uis| {
            let is_acceleration = self.selection == Some(PlotItem::Acceleration);
            let is_drag = self.selection == Some(PlotItem::Drag);

            let accel_clicked = uis[0]
                .add(Button::selectable(
                    is_acceleration,
                    (Atom::grow(), "Acceleration", Atom::grow()),
                ))
                .clicked();
            if accel_clicked {
                self.selection = if is_acceleration {
                    None
                } else {
                    Some(PlotItem::Acceleration)
                };
            }

            let drag_clicked = uis[1]
                .add(Button::selectable(
                    is_drag,
                    (Atom::grow(), "Drag", Atom::grow()),
                ))
                .clicked();
            if drag_clicked {
                self.selection = if is_drag { None } else { Some(PlotItem::Drag) };
            }
        });

        match self.selection {
            Some(PlotItem::Acceleration) => {
                UiFunctionEdit::new("Accleration", ("Speed", "Acceleration")).ui(
                    ui,
                    ui.id().with("acceleration"),
                    &mut train_performance.acceleration,
                );
            }
            Some(PlotItem::Drag) => {
                UiFunctionEdit::new("Drag", ("Speed", "Negative acceleration")).ui(
                    ui,
                    ui.id().with("drag"),
                    &mut train_performance.drag,
                );
            }
            None => {}
        }
    }

    #[expect(clippy::unused_self)]
    fn ui_form(
        &self,
        ui: &mut egui::Ui,
        _action: &mut AppAction,
        train_performance: &mut TrainPerformance,
    ) {
        Grid::new(ui.id().with("train_performance_brake"))
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label("Power steps");
                ui.add(
                    Slider::new(&mut train_performance.power_steps, 1..=8).drag_value_speed(0.1),
                );
                ui.end_row();

                ui.label("Brake steps");
                ui.add(
                    Slider::new(&mut train_performance.brake_steps, 1..=8).drag_value_speed(0.1),
                );
                ui.end_row();

                ui.label("Brake acceleration (km/h/s)");
                ui.add(
                    Slider::new(&mut train_performance.brake_acceleration, 0.0..=8.0)
                        .drag_value_speed(0.01),
                );
                ui.end_row();
            });
    }
}
