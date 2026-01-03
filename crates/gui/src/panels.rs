use crate::TunerApp;
use cli::Visualizer;

impl TunerApp {
    pub fn central_panel(&mut self, ctx: &egui::Context) {
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
                    ui.label("Click Start Microphone to begin");
                });
            }
        });
    }

    pub fn control_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("controls")
            .default_width(200.0)
            .show(ctx, |ui| {
                self.start_microphone_button(ui);
                ui.separator();
                self.features_button(ui);
            });
    }

    pub fn source_code_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("source code").show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.hyperlink_to("Source code", "https://github.com/LeMuffinMan/tuners");
                },
            );
        });
    }

    fn start_microphone_button(&mut self, ui: &mut egui::Ui) {
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
    }

    fn features_button(&mut self, ui: &mut egui::Ui) {
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
                .selectable_label(matches!(self.visualizer, Visualizer::WaveShape), "Waveform")
                .clicked()
            {
                self.visualizer = Visualizer::WaveShape;
            }
        }
    }

    pub fn mobile_visualizer(&mut self, ui: &mut egui::Ui) {
        let height = ui.available_height().min(180.0);

        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height),
            egui::Sense::hover(),
        );
        if self.rms_history.is_empty() {
            ui.label("RMS empty");
        } else {
            self.render_rms_in_rect(ui, rect);

        }
    }

    pub fn mobile_controls(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Controls");
                    ui.add_space(8.0);

                    self.start_microphone_button(ui);

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    self.features_button(ui);
                });
            });
    }
}
