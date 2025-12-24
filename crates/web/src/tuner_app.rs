use crate::AudioRingBuffer;
use crate::GLOBAL_RING;
use crate::start_audio;
use egui::{ Rect, Pos2, Color32, Vec2 };

pub enum UiType {
    Mobile,
    Desktop,
}

enum Visualizer {
    Freq,
    RMS,
    WaveShape,
}

pub struct TunerApp {
    ui_type : UiType, 
    visualizer: Visualizer,
    ringbuff: Option<AudioRingBuffer>,
    audio_start: bool,
}

impl TunerApp {
    pub fn new(ui_type: UiType) -> Self {
        Self {
            ui_type,
            ringbuff: None,
            visualizer: Visualizer::RMS,
            audio_start: false,
        }
    }

    pub fn start_audio(&mut self) {
        if self.audio_start {
            return;
        }

        self.audio_start = true;
        
        wasm_bindgen_futures::spawn_local(async {
            start_audio().await.unwrap();
        });
    }
}

impl TunerApp {
    fn get_rms(&mut self) -> f32 {
        if let Some(ringbuff) = &mut self.ringbuff {
            web_sys::console::log_1(&format!("Buffer len: {}", ringbuff.len()).into());
            let n = ringbuff.len();
            if n == 0 {
                return 0.0;
            }

            let mut tmp = vec![0.0; n];
            let read = ringbuff.peek_block(&mut tmp);
            if read == 0 {
                return 0.0;
            }

            // Diviser par le nombre d'échantillons lus, pas la capacité totale
            (tmp[..read].iter().map(|x| x * x).sum::<f32>() / read as f32).sqrt()
        } else {
            0.0
        }
    }

    fn render_rms(&mut self, ui: &mut egui::Ui) {
        let size = Vec2::new(24.0, 140.0);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();
        
        // fond gris
        painter.rect_filled(rect, 4.0, Color32::from_gray(30));

        let rms = self.get_rms();
        web_sys::console::log_1(&format!("RMS: {:.3}", rms).into());
        let db = rms_to_db(rms);
        // web_sys::console::log_1(&format!("RMS: {:.5}, dB: {:.2}", rms, db).into());
        let min_db = -60.0;
        let max_db = 0.0;

        let norm = ((db - min_db) / (max_db - min_db)).clamp(0.0, 1.0);
        let fill_height = rect.height() * norm;

        // barre remplie
        let filled_rect = Rect::from_min_max(
            Pos2::new(rect.left(), rect.bottom() - fill_height),
            Pos2::new(rect.right(), rect.bottom()),
        );
        painter.rect_filled(filled_rect, 4.0, Color32::from_rgb(0, 200, 0));
    }

}

impl eframe::App for TunerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {

            // ui.heading("🎵 Tuner WASM");
            match &self.ui_type {
                _ => {
                    egui::SidePanel::right("mode").show(ctx, |ui| {


                        if self.audio_start == false {
                            if ui.button("🎤 Activer le micro").clicked() {
                                self.start_audio();
                                self.ringbuff = GLOBAL_RING.with(|g| g.borrow_mut().take());
                            }
                        }


                        if ui.button("RMS").clicked() {
                            self.visualizer = Visualizer::RMS;
                        }    
                        if ui.button("Freq").clicked() {
                            self.visualizer = Visualizer::Freq;
                        }    
                        if ui.button("WaveShape").clicked() {
                            self.visualizer = Visualizer::WaveShape;
                        }    
                    });
                }
            }
            match self.visualizer {
                _ => {
                    self.render_rms(ui);
                }
                // RMS => {
                // },
                // Freq => {
                //
                // },
                // WaveShape => {
                //
                // },
            }
        });
    }
}

pub fn take_ringbuffer() -> Option<AudioRingBuffer> {
    GLOBAL_RING.with(|g| g.borrow_mut().take())
}

fn rms_to_db(rms: f32) -> f32 {
    20.0 * (rms.max(1e-9)).log10()
}

