use std::rc::Rc;
use std::cell::RefCell;
use audio::RingReader;
#[cfg(target_arch = "wasm32")]
use audio::backend::wasm::start_audio_wasm;
#[cfg(target_arch = "wasm32")]
use audio::backend::wasm::read_global_rms;
#[cfg(target_arch = "wasm32")]
use web_sys;

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
    pub ui_type : UiType, 
    visualizer: Visualizer,
    pub ring_reader: Option<Rc<RefCell<dyn RingReader>>>,
    pub audio_start: bool,
    pub rms_history: Vec<f32>,
}

impl TunerApp {
    pub fn set_ring_reader(&mut self, ring: Rc<RefCell<dyn RingReader>>) {
        self.ring_reader = Some(ring);
    }

    pub fn new(ui_type: UiType) -> Self {
        Self {
            ui_type,
            ring_reader: None,
            visualizer: Visualizer::RMS,
            audio_start: false,
            rms_history: Vec::new(),
        }
    }
}

impl TunerApp {
    //devrait aller dans DSP
    pub fn get_rms(&mut self) -> f32 {
        #[cfg(target_arch = "wasm32")]
        {
            read_global_rms()
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Pour le build natif, tu peux continuer à utiliser AudioRingBuffer si tu en as un
            if let Some(reader_rc) = &self.ring_reader {
                reader_rc.borrow_mut().get_rms()
            } else {
                0.0
            }
        }
    }

    fn start_audio(&mut self) {
        if self.audio_start {
            return;
        }

        self.audio_start = true;
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async {
                match start_audio_wasm().await {
                    Ok(_) => web_sys::console::log_1(&"Micro captured".into()),
                    Err(e) => web_sys::console::error_1(&e),
                }
            });
        }

        // Pour le build natif, tu pourrais appeler start_audio_native() ou autre
        #[cfg(not(target_arch = "wasm32"))]
        {
            // crate::audio::start_audio_native();
        }
    }

}

impl eframe::App for TunerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {

            // ui.heading("Tuner WASM");
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
                                #[cfg(target_arch = "wasm32")]
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

