
use crate::TunerApp;
use egui::{ Rect, Pos2, Color32 };

impl TunerApp {
    pub fn render_rms(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let width = size.x;
        let height = size.y;

        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();

        painter.rect_filled(rect, 0.0, Color32::from_gray(30));

        if self.rms_history.len() > width as usize {
            self.rms_history.remove(0); 
        }

        let n = self.rms_history.len();
        for (i, &v) in self.rms_history.iter().enumerate() {
            let bar_height = v * height;
            let x = rect.right() - n as f32 + i as f32; 
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x, rect.bottom() - bar_height),
                    Pos2::new(x + 1.0, rect.bottom()),
                ),
                0.0,
                Color32::from_rgb(0, 200, 0),
            );
        }
    }
}
