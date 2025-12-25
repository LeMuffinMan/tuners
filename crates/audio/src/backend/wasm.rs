use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ring::AudioRingBuffer;
use crate::ring::GLOBAL_RING;

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

