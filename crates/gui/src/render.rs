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

        // Fond noir
        painter.rect_filled(rect, 0.0, Color32::from_gray(30));

        // Ligne centrale (zéro)
        let center_y = rect.center().y;
        painter.line_segment(
            [
                Pos2::new(rect.left(), center_y),
                Pos2::new(rect.right(), center_y),
            ],
            Stroke::new(1.0, Color32::from_gray(60)),
        );

        // Obtenir les samples du DSP
        if let Some(dsp) = &self.dsp {
            let samples = dsp.get_samples(width as usize);

            if samples.len() > 1 {
                let mut points = Vec::new();

                for (i, &sample) in samples.iter().enumerate() {
                    let x = rect.left() + (i as f32 / samples.len() as f32) * width;
                    // Inverser et normaliser le sample (-1.0 à 1.0 devient haut à bas)
                    let y = center_y - (sample * height * 0.4);
                    points.push(Pos2::new(x, y));
                }

                // Dessiner la forme d'onde
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
                    
                    // let target_freq = Self::note_to_freq(&note);
                    // let cents = 1200.0 * (freq / target_freq).log2();
                    // 
                    // let gauge_width = rect.width() * 0.8;
                    // let gauge_height = 20.0;
                    // let gauge_center = Pos2::new(center.x, center.y + 140.0);
                    // 
                    // painter.rect_filled(
                    //     Rect::from_center_size(gauge_center, egui::vec2(gauge_width, gauge_height)),
                    //     5.0,
                    //     Color32::from_gray(50),
                    // );
                    // 
                    // for cents_mark in [-50, 0, 50] {
                    //     let x = gauge_center.x + (cents_mark as f32 / 50.0) * (gauge_width / 2.0);
                    //     painter.line_segment(
                    //         [Pos2::new(x, gauge_center.y - 15.0), Pos2::new(x, gauge_center.y + 15.0)],
                    //         Stroke::new(2.0, Color32::from_gray(100)),
                    //     );
                    // }
                    // 
                    // let indicator_x = gauge_center.x + (cents.clamp(-50.0, 50.0) / 50.0) * (gauge_width / 2.0);
                    // let color = if cents.abs() < 5.0 {
                    //     Color32::from_rgb(0, 255, 100) 
                    // } else if cents.abs() < 20.0 {
                    //     Color32::from_rgb(255, 200, 0)
                    // } else {
                    //     Color32::from_rgb(255, 50, 50) 
                    // };
                    // 
                    // painter.circle_filled(Pos2::new(indicator_x, gauge_center.y), 12.0, color);
                    // 
                    // let cents_text = if cents > 0.0 {
                    //     format!("+{:.0} cents", cents)
                    // } else {
                    //     format!("{:.0} cents", cents)
                    // };
                    // 
                    // painter.text(
                    //     Pos2::new(center.x, gauge_center.y + 40.0),
                    //     egui::Align2::CENTER_CENTER,
                    //     cents_text,
                    //     egui::FontId::proportional(24.0),
                    //     color,
                    // );
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

    fn note_to_freq(note: &str) -> f32 {
        let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        
        let note_name = if note.len() > 2 {
            &note[0..2]
        } else {
            &note[0..1]
        };
        
        let octave: i32 = note[note_name.len()..].parse().unwrap_or(4);
        
        let note_index = notes.iter().position(|&n| n == note_name).unwrap_or(9) as i32;
        
        let a4 = 440.0;
        let half_steps = (octave - 4) * 12 + note_index - 9;
        
        a4 * 2_f32.powf(half_steps as f32 / 12.0)
    }
}
