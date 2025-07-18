use super::{PlotAutoColor, UiPlotEdit};
use crate::{
    app::AppAction,
    state::TrainPerformance,
    ui::{UiFunctionEdit, ui_plot_edit::PlotEditEntry},
};
use egui::{Grid, ScrollArea, Sides, Slider, Window};
use egui_plot::{AxisHints, Plot};

#[derive(Debug, Clone, PartialEq, Eq)]
enum PerformanceTab {
    Acceleration,
    Brake,
}

#[derive(Debug)]
pub struct UiPerformanceWindow {
    tab: PerformanceTab,
    acceleration_plot: UiPlotEdit<()>,
    selection: Option<()>,
}

impl Default for UiPerformanceWindow {
    fn default() -> Self {
        Self {
            tab: PerformanceTab::Acceleration,
            acceleration_plot: UiPlotEdit::default(),
            selection: None,
        }
    }
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
            .default_size([500.0, 400.0])
            .min_size([300.0, 300.0])
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.tab,
                            PerformanceTab::Acceleration,
                            "Acceleration",
                        );
                        ui.selectable_value(&mut self.tab, PerformanceTab::Brake, "Brake");
                    });

                    ui.separator();

                    match self.tab {
                        PerformanceTab::Acceleration => {
                            self.ui_acceleration(ui, action, train_performance);
                        }
                        PerformanceTab::Brake => {
                            self.ui_brake(ui, action, train_performance);
                        }
                    }

                    ui.allocate_space(ui.available_size());
                });
            });
    }

    fn ui_acceleration(
        &mut self,
        ui: &mut egui::Ui,
        action: &mut AppAction,
        train_performance: &mut TrainPerformance,
    ) {
        Grid::new(ui.id().with("train_performance_acceleration"))
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label("Power steps");
                ui.add(
                    Slider::new(&mut train_performance.power_steps, 1..=8).drag_value_speed(0.1),
                );
                ui.end_row();
            });

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
            &mut [PlotEditEntry::new(
                &mut train_performance.acceleration,
                PlotAutoColor::get_color(0),
                "Acceleration".to_owned(),
                (),
            )],
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

        UiFunctionEdit::new("Accleration", ("Speed", "Acceleration"))
            .ui(ui, &mut train_performance.acceleration);
    }

    #[expect(clippy::unused_self)]
    fn ui_brake(
        &self,
        ui: &mut egui::Ui,
        _action: &mut AppAction,
        train_performance: &mut TrainPerformance,
    ) {
        Grid::new(ui.id().with("train_performance_brake"))
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
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
