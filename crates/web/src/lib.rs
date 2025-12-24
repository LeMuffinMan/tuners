use eframe::{App, egui};
use tuner_dsp::{autocorrelation, freq_to_tune};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use egui::ProgressBar;
use egui::{ Rect, Pos2, Color32, Vec2 };
use audio::ring::AudioRingBuffer;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{
    AudioContext,
    AudioWorkletNode,
    MediaStream,
    MediaStreamConstraints,
    MessageEvent,
};
use wasm_bindgen_futures::JsFuture;

thread_local! {
    static GLOBAL_RING: RefCell<Option<AudioRingBuffer>> = RefCell::new(None);
}

#[wasm_bindgen]
pub async fn start_audio() -> Result<(), JsValue> {
    use web_sys::*;

    let audio_ctx = AudioContext::new()?;

    let stream = wasm_bindgen_futures::JsFuture::from(
        window()
            .unwrap()
            .navigator()
            .media_devices()?
            .get_user_media_with_constraints(
                MediaStreamConstraints::new().audio(&JsValue::TRUE),
            )?,
    )
    .await?
    .dyn_into::<MediaStream>()?;

    let source = audio_ctx.create_media_stream_source(&stream)?;

    let worklet = audio_ctx.audio_worklet()?;
    JsFuture::from(worklet.add_module("my-processor.js")?).await?;

    let worklet = AudioWorkletNode::new(&audio_ctx, "my-processor")?;
    source.connect_with_audio_node(&worklet)?;

    let ring = AudioRingBuffer::new(48_000 * 2); // ~2 secondes
    GLOBAL_RING.with(|g| *g.borrow_mut() = Some(ring));


    let closure = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        let array = js_sys::Float32Array::new(&e.data());
        
        GLOBAL_RING.with(|g| {
            if let Some(ring) = g.borrow_mut().as_mut() {
                let mut tmp = vec![0.0; array.length() as usize];
                array.copy_to(&mut tmp);
                ring.push_samples(&tmp);
            }
        });
    });

    worklet.port().unwrap().set_onmessage(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

    web_sys::console::log_1(&"🎤 Micro capturé (WASM)".into());
    Ok(())
}

pub fn take_ringbuffer() -> Option<AudioRingBuffer> {
    GLOBAL_RING.with(|g| g.borrow_mut().take())
}


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
            let n = ringbuff.len();
            if n == 0 {
                return 0.0;
            }

            let mut tmp = vec![0.0; n];
            let read = ringbuff.pop_block(&mut tmp);
            if read == 0 {
                return 0.0;
            }
            (tmp[..read].iter().map(|x| x * x).sum::<f32>() / ringbuff.len() as f32).sqrt()
        } else {
            0.0
        }
    }

    fn render_rms(&mut self, ui: &mut egui::Ui) {
        let size = Vec2::new(24.0, 140.0);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, Color32::from_gray(30));

        let rms = self.get_rms();
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
        if self.ringbuff.is_none() {
            self.ringbuff = take_ringbuffer();
        }
        egui::CentralPanel::default().show(ctx, |ui| {

            // ui.heading("🎵 Tuner WASM");
            match &self.ui_type {
                _ => {
                    egui::SidePanel::right("mode").show(ctx, |ui| {


                        if ui.button("🎤 Activer le micro").clicked() {
                            self.start_audio();
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

fn rms_to_db(rms: f32) -> f32 {
    20.0 * (rms.max(1e-9)).log10()
}

fn start_audio_async() {
    wasm_bindgen_futures::spawn_local(async {
        start_audio().await;
        web_sys::console::log_1(&"Microphone ready".into());
    });
}

pub fn get_ui_type(window: web_sys::Window) -> UiType {
    let ua = window.navigator().user_agent().unwrap_or_default();
    let ui_type = ua.to_lowercase().contains("mobi") || window.inner_width().unwrap().as_f64().unwrap_or(1024.0) < 800.0;
    match ui_type {
        true => UiType::Mobile,
        false => UiType::Desktop,
    }
}
