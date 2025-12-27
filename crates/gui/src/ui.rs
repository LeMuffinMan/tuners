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
    visualizer: Visualizer,
    pub audio_start: bool,
    pub rms_history: Vec<f32>,
    #[cfg(target_arch = "wasm32")]
    audio_initializing: bool,
}

impl eframe::App for TunerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        if self.audio_start {
            if let Some(dsp) = &mut self.dsp {
                dsp.update();
                let rms = dsp.get_rms();
                self.rms_history.push(rms);
            } else {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&"DSP is None!".into());
            }
        }
        egui::TopBottomPanel::bottom("source code").show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.hyperlink_to("Source code", "https://github.com/LeMuffinMan/tuners");
                },
            );
        });
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

                    if ui
                        .selectable_label(matches!(self.visualizer, Visualizer::RMS), "RMS")
                        .clicked()
                    {
                        self.visualizer = Visualizer::RMS;
                    }

                    if ui
                        .selectable_label(matches!(self.visualizer, Visualizer::Freq), "Frequency")
                        .clicked()
                    {
                        self.visualizer = Visualizer::Freq;
                    }

                    if ui
                        .selectable_label(
                            matches!(self.visualizer, Visualizer::WaveShape),
                            "Waveform",
                        )
                        .clicked()
                    {
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
                        ui.vertical_centered(|_ui| {});
                    }
                    Visualizer::WaveShape => {
                        ui.vertical_centered(|_ui| {});
                    }
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Tune.rs");
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

    //comment evacuer ce code duplique ?
    #[cfg(not(target_arch = "wasm32"))]
    fn start_audio(&mut self) {
        if self.audio_start {
            return;
        }

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
    fn start_audio(&mut self) {
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
