#[derive(Debug, Default)]
pub struct PlotAutoColor {
    index: usize,
}

impl PlotAutoColor {
    #[expect(unused)]
    pub fn next_color(&mut self) -> egui::Color32 {
        let i = self.index;
        self.index += 1;
        Self::get_color(i)
    }

    pub fn get_color(index: usize) -> egui::Color32 {
        let golden_ratio = (5f32.sqrt() - 1.0) / 2.0;
        let h = index as f32 * golden_ratio;
        egui::epaint::Hsva::new(h, 0.85, 0.5, 1.0).into()
    }
}
