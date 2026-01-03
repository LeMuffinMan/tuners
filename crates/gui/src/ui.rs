use audio::audio_bridge::{AudioBridge, SAMPLE_RATE};
use audio::backend::AudioBackend;
#[cfg(not(target_arch = "wasm32"))]
use audio::backend::native;
#[cfg(target_arch = "wasm32")]
use audio::backend::wasm;
use cli::Visualizer;
use dsp::DigitalSignalProcessor;
#[cfg(target_arch = "wasm32")]
use web_sys;

pub enum DeviceType {
    Mobile,
    Desktop,
}

pub struct TunerApp {
    pub dsp: Option<DigitalSignalProcessor>,
    pub ui_type: DeviceType,
    #[cfg(not(target_arch = "wasm32"))]
    backend: Option<native::NativeAudioBackend>,
    pub visualizer: Visualizer,
    pub audio_start: bool,
    pub rms_history: Vec<f32>,
    #[cfg(target_arch = "wasm32")]
    pub audio_initializing: bool, //to fix for promise / future return
}

///at each frame, we update the dsp, and display panels.
impl eframe::App for TunerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        match self.ui_type {
            DeviceType::Desktop => {
                if self.audio_start {
                    self.update_dsp();
                }
                self.source_code_panel(ctx);
                self.control_panel(ctx);
                self.central_panel(ctx);
            }
            DeviceType::Mobile => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let available = ui.available_size();
                    ui.vertical_centered(|ui| {
                        ui.set_max_width(available.x.min(420.0));
                        egui::Frame::new()
                            .inner_margin(egui::Margin::same(12))
                            .show(ui, |_ui| {
                                self.source_code_panel(ctx);
                                self.control_panel(ctx);
                                self.central_panel(ctx);
                            });
                    });
                });
            }
        }
        if self.audio_start {
            ctx.request_repaint();
        }
    }
}

impl TunerApp {
    pub fn new(ui_type: DeviceType) -> Self {
        Self {
            dsp: None,
            ui_type,
            #[cfg(not(target_arch = "wasm32"))]
            backend: None,
            visualizer: Visualizer::RMS,
            audio_start: false,
            rms_history: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            audio_initializing: false,
        }
    }

    pub fn update_dsp(&mut self) {
        if let Some(dsp) = &mut self.dsp {
            dsp.update();
            let rms = dsp.get_rms();
            self.rms_history.push(rms);
        } else {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&"DSP is None!".into());
        }
    }

    //comment evacuer ce code duplique ?
    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_audio(&mut self) {
        if self.audio_start {
            return;
        }

        //we set our ringbuff to contain 2 seconds of audio, sampled at SAMPLE_RATE
        let (bridge, producer) = AudioBridge::new(SAMPLE_RATE as usize * 2);
        self.dsp = Some(DigitalSignalProcessor::new(bridge.consumer));

        match native::NativeAudioBackend::new(producer) {
            Ok(mut backend) => match backend.start() {
                Ok(_) => {
                    self.backend = Some(backend);
                    self.audio_start = true;
                }
                Err(e) => {
                    eprintln!("Failed to start audio: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Failed to create audio backend: {}", e);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn start_audio(&mut self) {
        if self.audio_start || self.audio_initializing {
            web_sys::console::log_1(&"Already started or initializing".into());
            return;
        }

        web_sys::console::log_1(&"Starting audio...".into());
        self.audio_initializing = true;

        let (bridge, producer) = AudioBridge::new(SAMPLE_RATE as usize * 2);
        web_sys::console::log_1(
            &format!("Bridge created, buffer size: {}", SAMPLE_RATE * 2).into(),
        );

        self.dsp = Some(DigitalSignalProcessor::new(bridge.consumer));
        web_sys::console::log_1(&"DSP created".into());

        wasm_bindgen_futures::spawn_local(async move {
            web_sys::console::log_1(&"Async task started".into());
            match wasm::WasmAudioBackend::new(producer).await {
                Ok(mut backend) => {
                    web_sys::console::log_1(&"Backend created successfully".into());

                    match backend.start() {
                        Ok(_) => {
                            web_sys::console::log_1(&"Backend started successfully".into());
                            std::mem::forget(backend);
                        }
                        Err(e) => {
                            web_sys::console::error_1(
                                &format!("Failed to start backend: {}", e).into(),
                            );
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to create backend: {}", e).into());
                }
            }
        });

        //set audio_start au retour de la promise / future
        self.audio_start = true;
        self.audio_initializing = false;
        web_sys::console::log_1(&"Audio marked as started".into());
    }

    pub fn stop_audio(&mut self) {
        if !self.audio_start {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(backend) = &mut self.backend {
                backend.stop();
            }
            self.backend = None;
        }

        //au click sur start mic on devrait relancer l'audio, ou garder le backend et le reprendre
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Wasm backend unplugged".into());

        self.dsp = None;
        self.audio_start = false;
        self.rms_history.clear();
    }
}
