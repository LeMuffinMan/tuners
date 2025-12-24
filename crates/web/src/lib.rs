use eframe::{App, egui};
use tuner_dsp::{autocorrelation, freq_to_tune};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use egui::ProgressBar;
use egui::{ Rect, Pos2, Color32, Vec2 };



#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("tunersappid")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    let runner = eframe::WebRunner::new();
    runner
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(|_cc| Ok(Box::new(TunerApp::new(get_ui_type(window))) as Box<dyn App>)),
        )
        .await
}

enum UiType {
    Mobile,
    Desktop,
}

enum Visualizer {
    Freq,
    RMS,
    WaveShape,
}

struct TunerApp {
    ui_type : UiType, 
    visualizer: Visualizer,
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl TunerApp {
    pub fn new(ui_type: UiType, buffer: Arc<Mutex<Vec<f32>>>) -> Self {
        Self {
            ui_type,
            buffer,
            visualizer: Visualizer::Freq
        }
    }
}

impl TunerApp {
    fn get_rms() -> f32 {
        (self.buffer.iter().map(|x| x * x).sum::<f32>() / buffer.len() as f32).sqrt()
    }

    fn render_rms(&mut self, ui: &mut egui::Ui) {
        let size = Vec2::new(24.0, 140.0);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, Color32::from_gray(30));

        //let rms = self.get_rms();
        let rms = 0.0;
        //Ici on exploite le buffer partage
        let db = rms_to_db(rms);
        let min_db = -60.0;
        let max_db = 0.0;

        let norm = ((db / min_db) / (max_db - min_db)).clamp(0.0, 1.0);

        let fill_height = rect.height() * norm;
        let fill_rect = Rect::from_min_max(
            Pos2::new(rect.left(), rect.bottom() - fill_height),
            Pos2::new(rect.right(), rect.bottom()),
        );
    }
}

impl eframe::App for TunerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {

            // ui.heading("🎵 Tuner WASM");
            match &self.ui_type {
                _ => {
                    egui::SidePanel::right("mode").show(ctx, |ui| {
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

fn rms_to_db(rms: f32) -> f32 {
    20.0 * (rms.max(1e-9)).log10()
}

pub fn get_ui_type(window: web_sys::Window) -> UiType {
    let ua = window.navigator().user_agent().unwrap_or_default();
    let ui_type = ua.to_lowercase().contains("mobi") || window.inner_width().unwrap().as_f64().unwrap_or(1024.0) < 800.0;
    match ui_type {
        true => UiType::Mobile,
        false => UiType::Desktop,
    }
}
