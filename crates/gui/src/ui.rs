#[cfg(target_arch = "wasm32")]
use web_sys;
#[cfg(target_arch = "wasm32")]
use audio::backend::wasm;
#[cfg(not(target_arch = "wasm32"))]
use audio::backend::native;
use audio::ring::{SAMPLE_RATE, BUFFER_SIZE, AudioBridge};
use audio::backend::AudioBackend;
use rtrb::Consumer;
use clap::ValueEnum;

pub enum UiType {
    Mobile,
    Desktop,
}

//crate dediee ? pour clap ?
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Visualizer {
    Freq,
    RMS,
    WaveShape,
}

pub struct DigitalSignalProcessor {
    consumer: Consumer<f32>,
    pub rms: f32,
    sample_buffer: Vec<f32>,
}

//Tout va dans la crate DSP
impl DigitalSignalProcessor {
    pub fn new(consumer: Consumer<f32>) -> Self {
        Self {
            consumer,
            sample_buffer: Vec::with_capacity(BUFFER_SIZE),
            rms: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.sample_buffer.clear();

        let mut count = 0;
        while let Ok(sample) = self.consumer.pop() {
            self.sample_buffer.push(sample);
            count += 1;
            if self.sample_buffer.len() >= BUFFER_SIZE {
                break;
            }
        }

        //faire une macro pour les logs pour egui / cli / wasm 
        if count > 0 {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("Read {} samples from ringbuffer", count).into());
        }

        if self.sample_buffer.is_empty() {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&"No samples available".into());
            self.rms = 0.0; 
            return; 
        };

        let sum: f32 = self.sample_buffer.iter().map(|&s| s * s).sum();
        self.rms = (sum / self.sample_buffer.len() as f32).sqrt();

        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("RMS calculated: {}", self.rms).into());
    }

    pub fn get_rms(&self) -> f32 {
        self.rms
    }
}

pub struct TunerApp {
    pub dsp: Option<DigitalSignalProcessor>,
    pub ui_type : UiType, 
    #[cfg(not(target_arch = "wasm32"))]
    backend: Option<native::NativeAudioBackend>,
    visualizer: Visualizer,
    pub audio_start: bool,
    pub rms_history: Vec<f32>,
    #[cfg(target_arch = "wasm32")]
    audio_initializing: bool,
}

impl TunerApp {

    pub fn new(ui_type: UiType) -> Self {
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

    #[cfg(not(target_arch = "wasm32"))]
    fn start_audio(&mut self) {
        if self.audio_start {
            return;
        }

        let (bridge, producer) = AudioBridge::new(SAMPLE_RATE as usize * 2);
        self.dsp = Some(DigitalSignalProcessor::new(bridge.consumer));

        match native::NativeAudioBackend::new(producer) {
            Ok(mut backend) => {
                match backend.start() {
                    Ok(_) => {
                        self.backend = Some(backend);
                        self.audio_start = true;
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
    fn start_audio(&mut self) {
        if self.audio_start || self.audio_initializing {
            web_sys::console::log_1(&"Already started or initializing".into());
            return;
        }

        web_sys::console::log_1(&"Starting audio...".into());
        self.audio_initializing = true;

        let (bridge, producer) = AudioBridge::new(SAMPLE_RATE as usize * 2);
        web_sys::console::log_1(&format!("Bridge created, buffer size: {}", SAMPLE_RATE * 2).into());

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
                            web_sys::console::error_1(&format!("Failed to start backend: {}", e).into());
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to create backend: {}", e).into());
                }
            }
        });

        self.audio_start = true;
        self.audio_initializing = false;
        web_sys::console::log_1(&"Audio marked as started".into());
    }

    fn stop_audio(&mut self) {
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

        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Audio stopped (cleanup limited in WASM)".into());

        self.dsp = None;
        self.audio_start = false;
        self.rms_history.clear();
    }
}

impl eframe::App for TunerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        if self.audio_start {
            #[cfg(target_arch = "wasm32")]
            if self.rms_history.len() % 60 == 0 {
                web_sys::console::log_1(&format!("Audio active, history size: {}", self.rms_history.len()).into());
            }
            
            if let Some(dsp) = &mut self.dsp {
                dsp.update();
                let rms = dsp.get_rms();
                self.rms_history.push(rms);
                
                #[cfg(target_arch = "wasm32")]
                if self.rms_history.len() % 60 == 0 {
                    web_sys::console::log_1(&format!("Latest RMS: {}", rms).into());
                }
            } else {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&"DSP is None!".into());
            }
        }

        egui::SidePanel::left("controls")
            .default_width(200.0)
            .show(ctx, |ui| {
                if !self.audio_start {
                    #[cfg(target_arch = "wasm32")]
                    {
                        if self.audio_initializing {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Starting...");
                            });
                        } else if ui.button("🎤 Start Microphone").clicked() {
                            self.start_audio();
                        }
                    }
                    
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if ui.button("Start Microphone").clicked() {
                            self.start_audio();
                        }
                    }
                } else {
                    if ui.button("Stop Microphone").clicked() {
                        self.stop_audio();
                    }
                    
                    ui.label("Recording");
                }
                
                ui.separator();
                
                if self.audio_start {
                    ui.label("Visualizer:");
                    
                    if ui.selectable_label(
                        matches!(self.visualizer, Visualizer::RMS),
                        "RMS"
                    ).clicked() {
                        self.visualizer = Visualizer::RMS;
                    }
                    
                    if ui.selectable_label(
                        matches!(self.visualizer, Visualizer::Freq),
                        "Frequency"
                    ).clicked() {
                        self.visualizer = Visualizer::Freq;
                    }
                    
                    if ui.selectable_label(
                        matches!(self.visualizer, Visualizer::WaveShape),
                        "Waveform"
                    ).clicked() {
                        self.visualizer = Visualizer::WaveShape;
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.audio_start {
                match self.visualizer {
                    Visualizer::RMS => {
                        self.render_rms(ui);
                    }
                    Visualizer::Freq => {
                        ui.vertical_centered(|_ui| {
                        });
                    }
                    Visualizer::WaveShape => {
                        ui.vertical_centered(|_ui| {
                        });
                    }
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Tuners");
                    ui.add_space(20.0);
                    ui.label("Click 'Start Microphone' to begin");
                });
            }
        });

        if self.audio_start {
            ctx.request_repaint();
        }
    }
}
