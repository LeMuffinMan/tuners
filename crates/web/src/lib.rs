use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use eframe::App;
use std::cell::RefCell;
use std::rc::Rc;

// use tuner_dsp::{autocorrelation, freq_to_tune};
mod tuner_app;
use tuner_app::{UiType, TunerApp};
use audio::ring::AudioRingBuffer;

thread_local! {
///This gives us a mutable global. This buffer is filled by audio side, and read by ui
///Rc and RefCell let us use the buffer as writer and reader in the same WASM thread, 
///checking at runtime to borrow it as mutable. 
///Panic in case of error
    static GLOBAL_RING: RefCell<Option<Rc<RefCell<AudioRingBuffer>>>> = RefCell::new(None);
}

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

fn get_ui_type(window: web_sys::Window) -> UiType {
    let ua = window.navigator().user_agent().unwrap_or_default();
    let ui_type = ua.to_lowercase().contains("mobi") || window.inner_width().unwrap().as_f64().unwrap_or(1024.0) < 800.0;
    match ui_type {
        true => UiType::Mobile,
        false => UiType::Desktop,
    }
}

///We expose this function to JS to init audio input
#[wasm_bindgen]
pub async fn start_audio() -> Result<(), JsValue> {
    use web_sys::*;

    let audio_ctx = AudioContext::new()?;

    //we only want microphone input
    let constraints = {
        let c = MediaStreamConstraints::new();
        c.set_audio(&JsValue::TRUE);
        c
    };

    //This async func waits, through js, authorization for microphone access
    let stream = wasm_bindgen_futures::JsFuture::from(
        window()
            .unwrap()
            .navigator()
            .media_devices()?
            .get_user_media_with_constraints(&constraints)?,
    )
    .await?
    .dyn_into::<MediaStream>()?;

    //we create a node to connect with our AudioWorklet
    let source = audio_ctx.create_media_stream_source(&stream)?;

    //this load an audioworklet from our declaration in my-processor, 
    let worklet = audio_ctx.audio_worklet()?;
    JsFuture::from(worklet.add_module("my-processor.js")?).await?;

    //now we can connect the microphone to our audio processor
    let worklet = AudioWorkletNode::new(&audio_ctx, "my-processor")?;
    source.connect_with_audio_node(&worklet)?;

    //We now initialise the ring buffer : 2 seconds, 48 kHz 
    let ring = Rc::new(RefCell::new(AudioRingBuffer::new(48_000 * 2)));
    GLOBAL_RING.with(|g| *g.borrow_mut() = Some(ring.clone()));

    //We define a closure, that we will call for each message from our audioworklet 
    let closure = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        let array = js_sys::Float32Array::new(&e.data());
        //array is the received data from js, we open a closure to write in the ring buffer.
        //This function being async, we can safely read in this ring buffer by opening another
        //closure and borrowing it 
        GLOBAL_RING.with(|g| {
            if let Some(ring_rc) = g.borrow().as_ref() {
                let mut ring = ring_rc.borrow_mut(); // <-- emprunt mutable
                let mut tmp = vec![0.0; array.length() as usize];
                array.copy_to(&mut tmp);
                ring.push_samples(&tmp);
                // web_sys::console::log_1(&format!("Pushed {} samples, buffer len={}", tmp.len(), ring.len()).into());
            }
        });
    });

    //when we receive a message from the worklet, we execute our closure and the message 
    worklet.port().unwrap().set_onmessage(Some(closure.as_ref().unchecked_ref()));
    //we want to prevent Rust to destroy the closure, or js will crash
    closure.forget();

    web_sys::console::log_1(&"Micro captured".into());
    Ok(())
}

