use eframe::App;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

// use tuner_dsp::{autocorrelation, freq_to_tune};
use gui::{DeviceType, TunerApp};

///This is our end point, we init the canvas and the runner to run our UI
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

fn get_ui_type(window: web_sys::Window) -> DeviceType {
    let ua = window.navigator().user_agent().unwrap_or_default();
    let ui_type = ua.to_lowercase().contains("mobi")
        || window.inner_width().unwrap().as_f64().unwrap_or(1024.0) < 800.0;
    match ui_type {
        true => DeviceType::Mobile,
        false => DeviceType::Desktop,
    }
}
