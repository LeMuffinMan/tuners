use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ring::DSPRingBuffer;

thread_local! {
    pub static GLOBAL_RING: RefCell<Option<Rc<RefCell<DSPRingBuffer>>>> = RefCell::new(None);
}

// Fonction publique unifiée
pub fn start_audio() {
    spawn_local(async {
        start_audio_wasm().await.unwrap();
    });
}

///We expose this function to JS to init audio input
#[wasm_bindgen]
pub async fn start_audio_wasm() -> Result<(), JsValue> {
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
    let ring = Rc::new(RefCell::new(DSPRingBuffer::new(48_000 * 2)));
    GLOBAL_RING.with(|g| *g.borrow_mut() = Some(ring.clone()));

    //We define a closure, that we will call for each message from our audioworklet 
    let closure = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        // Récupération du RMS depuis l'objet JS { rms: ... }
        let rms = js_sys::Reflect::get(&e.data(), &JsValue::from_str("rms"))
            .unwrap()
            .as_f64()
            .unwrap() as f32;

        // Log dans la console
        web_sys::console::log_1(&format!("RMS: {}", rms).into());

        // Stockage dans le ring buffer
        GLOBAL_RING.with(|g| {
            if let Some(ring_rc) = g.borrow().as_ref() {
                let mut ring = ring_rc.borrow_mut();
                ring.push_rms(rms);
            }
        });
    });

    // Associer le closure à l'AudioWorkletNode
    worklet.port().unwrap().set_onmessage(Some(closure.as_ref().unchecked_ref()));

    // Éviter que Rust drop le closure
    closure.forget();
    web_sys::console::log_1(&"Micro captured".into());
    Ok(())
}

