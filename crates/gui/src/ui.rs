use audio::audio_bridge::AudioBridge;
use audio::backend::AudioBackend;
#[cfg(not(target_arch = "wasm32"))]
use audio::backend::native;
#[cfg(target_arch = "wasm32")]
use audio::backend::wasm;
use dsp::Visualizer;
use dsp::DigitalSignalProcessor;
use egui::FontId;
use egui::TextStyle;
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
        if self.audio_start {
            self.update_dsp();
        }
        match self.ui_type {
            DeviceType::Desktop => {
                self.source_code_panel(ctx);
                self.control_panel(ctx);
                self.central_panel(ctx);
            }
            DeviceType::Mobile => {
                self.apply_styles(ctx);
                self.source_code_panel(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    let max_width = ui.available_width().min(420.0);

                    ui.set_max_width(max_width);

                    self.mobile_controls(ui);
                    ui.add_space(12.0);

                    self.mobile_visualizer(ui);
                    ui.add_space(12.0);
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
            dsp.update(self.visualizer);
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
        let (bridge, producer) = AudioBridge::new();
        self.dsp = Some(DigitalSignalProcessor::new(bridge.consumer));
        match native::NativeAudioBackend::new(producer) {
            Ok(mut backend) => {
                let sample_rate = backend.sample_rate();
                println!("Backend sample rate: {} Hz", sample_rate);
                
                if let Some(ref mut dsp) = self.dsp {
                    dsp.sample_rate = sample_rate;
                }
                
                match backend.start() {
                    Ok(_) => {
                        self.backend = Some(backend);
                        self.audio_start = true;
                        println!("Audio started successfully");
                    }
                    Err(e) => {
                        eprintln!("Failed to start audio: {}", e);
                    }
                }
            }
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

        let sample_rate = match web_sys::AudioContext::new() {
            Ok(ctx) => {
                let sr = ctx.sample_rate();
                web_sys::console::log_1(&format!("Detected sample rate: {} Hz", sr).into());
                sr
            }
            Err(_) => {
                web_sys::console::warn_1(&"Failed to detect sample rate, using default 48000 Hz".into());
                48000.0 
            }
        };

        let (bridge, producer) = AudioBridge::new();
        web_sys::console::log_1(&format!("DSP created with sample rate: {} Hz", sample_rate).into());

        self.dsp = Some(DigitalSignalProcessor::new(bridge.consumer));
        web_sys::console::log_1(&"DSP created".into());

        wasm_bindgen_futures::spawn_local(async move {
            web_sys::console::log_1(&"Async task started".into());
            match wasm::WasmAudioBackend::new(producer).await {
                Ok(mut backend) => {
                    web_sys::console::log_1(&format!(
                        "Backend created with sample rate: {} Hz",
                        backend.sample_rate
                    ).into());
                    
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
    pub fn apply_styles(&mut self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(70.0, egui::FontFamily::Proportional),
            ),
            (
                TextStyle::Body,
                FontId::new(30.0, egui::FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(28.0, egui::FontFamily::Monospace),
            ),
            (
                TextStyle::Button,
                FontId::new(40.0, egui::FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(18.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();
        ctx.set_style(style);
    }
}
