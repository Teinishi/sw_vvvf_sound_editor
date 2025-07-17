use crate::editable_function::{EditableFunction, EditableFunctionMode};
use egui::{ComboBox, DragValue, Label, RichText, vec2};

#[derive(Debug, Default)]
pub struct UiFunctionEdit {
    percentage: (bool, bool),
}

impl UiFunctionEdit {
    pub fn new(percentage: (bool, bool)) -> Self {
        Self { percentage }
    }

    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        title: impl Into<egui::WidgetText>,
        func: &mut EditableFunction,
    ) {
        ui.horizontal(|ui| {
            ui.label(title);
            ComboBox::new(id.with("editable_function_mode_select"), "")
                .selected_text(format!("{:?}", func.mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut func.mode, EditableFunctionMode::Points, "Points");
                    ui.selectable_value(
                        &mut func.mode,
                        EditableFunctionMode::Expression,
                        "Expression",
                    );
                });
        });

        match func.mode {
            EditableFunctionMode::Points => {
                self.ui_points(ui, func);
            }
            EditableFunctionMode::Expression => {
                self.ui_expression(ui, func);
            }
        }
    }

    fn ui_points(&self, ui: &mut egui::Ui, func: &mut EditableFunction) {
        let mut changed = None;

        for (i, point) in func.points().iter().enumerate() {
            let mut x = point.0;
            let mut y = point.1;

            ui.horizontal(|ui| {
                let mut drag_x = DragValue::new(&mut x).speed(0.1).max_decimals(1);
                if self.percentage.0 {
                    drag_x = drag_x
                        .custom_formatter(|x, _| format!("{:.1}", 100.0 * x))
                        .custom_parser(|s| s.parse().ok().map(|v: f64| v / 100.0))
                        .suffix("%");
                }

                let mut drag_y = DragValue::new(&mut y).speed(0.01).max_decimals(3);
                if self.percentage.1 {
                    drag_y = drag_y
                        .custom_formatter(|x, _| format!("{:.1}", 100.0 * x))
                        .custom_parser(|s| s.parse().ok().map(|v: f64| v / 100.0))
                        .suffix("%");
                }

                let changed_x = ui.add_sized(vec2(60.0, 20.0), drag_x).changed();
                let changed_y = ui.add_sized(vec2(60.0, 20.0), drag_y).changed();
                if changed_x || changed_y {
                    changed = Some((i, (x, y)));
                }
            });
        }

        if let Some((i, pos)) = changed {
            func.move_point_to(i, pos);
        }
    }

    #[expect(clippy::unused_self)]
    fn ui_expression(&self, ui: &mut egui::Ui, func: &mut EditableFunction) {
        let mut text = func.expression().to_owned();

        ui.text_edit_singleline(&mut text);

        if let Some(err) = func.expression_err() {
            ui.add(Label::new(
                RichText::new(format!("{err}"))
                    .color(egui::Color32::RED)
                    .small(),
            ));
        }

        func.set_expression(&text);
    }
}
