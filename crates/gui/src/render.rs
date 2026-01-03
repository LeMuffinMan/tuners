use crate::TunerApp;

use egui::Stroke;
use egui::{Color32, Pos2, Rect};

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

    pub fn render_rms_in_rect(&mut self, ui: &egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        let width = rect.width();
        let height = rect.height();

        painter.rect_filled(rect, 0.0, Color32::from_gray(30));

        if self.rms_history.len() > width as usize {
            self.rms_history.remove(0);
        }

        let n = self.rms_history.len();
        for (i, &v) in self.rms_history.iter().enumerate() {
            let bar_height = (v * height).clamp(0.0, height);
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

    pub fn render_waveform(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        self.render_waveform_in_rect(ui, rect);
    }

    pub fn render_waveform_in_rect(&mut self, ui: &egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        let width = rect.width();
        let height = rect.height();

        painter.rect_filled(rect, 0.0, Color32::from_gray(30));

        let center_y = rect.center().y;
        painter.line_segment(
            [
                Pos2::new(rect.left(), center_y),
                Pos2::new(rect.right(), center_y),
            ],
            Stroke::new(1.0, Color32::from_gray(60)),
        );

        if let Some(dsp) = &self.dsp {
            let samples = dsp.get_samples(width as usize);

            if samples.len() > 1 {
                let mut points = Vec::new();

                for (i, &sample) in samples.iter().enumerate() {
                    let x = rect.left() + (i as f32 / samples.len() as f32) * width;
                    let y = center_y - (sample * height * 0.4);
                    points.push(Pos2::new(x, y));
                }

                painter.add(egui::Shape::line(
                    points,
                    Stroke::new(2.0, Color32::from_rgb(0, 200, 255)),
                ));
            }
        }
    }

    pub fn render_tuner(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        self.render_tuner_in_rect(ui, rect);
    }

    pub fn render_tuner_in_rect(&mut self, ui: &egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, Color32::from_gray(30));

        if let Some(dsp) = &self.dsp {
            let center = rect.center();

            if let Some(note) = dsp.get_note() {
                if let Some(freq) = dsp.get_frequency() {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        &note,
                        egui::FontId::proportional(120.0),
                        Color32::from_rgb(0, 255, 100),
                    );

                    painter.text(
                        Pos2::new(center.x, center.y + 80.0),
                        egui::Align2::CENTER_CENTER,
                        format!("{:.1} Hz", freq),
                        egui::FontId::proportional(40.0),
                        Color32::from_gray(200),
                    );
                }
            } else {
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "Play a note...",
                    egui::FontId::proportional(60.0),
                    Color32::from_gray(150),
                );
            }
        }
    }
}
