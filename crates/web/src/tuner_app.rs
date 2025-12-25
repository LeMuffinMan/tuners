use crate::AudioRingBuffer;
use crate::Rc;
use crate::RefCell;
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
    ringbuff: Option<Rc<RefCell<AudioRingBuffer>>>,
    audio_start: bool,
    rms_history: Vec<f32>, // <- nouvel ajout
    max_history: usize, 
}

impl TunerApp {
    pub fn init_ringbuffer(&mut self) {
        if self.ringbuff.is_none() {
            GLOBAL_RING.with(|g| {
                if let Some(ring) = g.borrow().as_ref() {
                    self.ringbuff = Some(ring.clone());
                }
            });
        }
    }

    pub fn new(ui_type: UiType) -> Self {
        Self {
            ui_type,
            ringbuff: None,
            visualizer: Visualizer::RMS,
            audio_start: false,
            rms_history: Vec::new(),
            max_history: 500,
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
        if let Some(ring_rc) = &self.ringbuff {
            let mut ring = ring_rc.borrow_mut(); // emprunt mutable
            let n = ring.len();
            if n == 0 { return 0.0; }

            let mut tmp = vec![0.0; n];
            let read = ring.peek_block(&mut tmp);
            if read == 0 { return 0.0; }

            (tmp[..read].iter().map(|x| x*x).sum::<f32>() / read as f32).sqrt()
        } else {
            0.0
        }
    }

    fn render_rms(&mut self, ui: &mut egui::Ui) {
        // Taille du panel (barre latérale)
        let size = ui.available_size();
        let width = size.x;
        let height = size.y;

        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();

        // fond gris
        painter.rect_filled(rect, 0.0, Color32::from_gray(30));

        // calcul RMS et normalisation
        let rms = self.get_rms();

        // ajouter à l'historique
        self.rms_history.push(rms * height / 200.0);
        if self.rms_history.len() > width as usize {
            self.rms_history.remove(0); // supprime la valeur la plus ancienne à gauche
        }

        let n = self.rms_history.len();
        for (i, &v) in self.rms_history.iter().enumerate() {
            let bar_height = v * height;
            let x = rect.right() - n as f32 + i as f32; // nouvelle valeur à droite
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

impl eframe::App for TunerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        self.init_ringbuffer();
        egui::CentralPanel::default().show(ctx, |ui| {

            // ui.heading("🎵 Tuner WASM");
            match &self.ui_type {
                _ => {
                    if self.audio_start {
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
                    }
                    egui::SidePanel::left("mode").show(ctx, |ui| {


                        if self.audio_start == false {
                            if ui.button("🎤 Activer le micro").clicked() {
                                self.start_audio();
                                // self.ringbuff = GLOBAL_RING.with(|g| g.borrow_mut().take());
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

        });
        ctx.request_repaint();
    }
}

fn rms_to_db(rms: f32) -> f32 {
    20.0 * (rms.max(1e-9)).log10()
}

