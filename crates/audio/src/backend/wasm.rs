use crate::backend::AudioBackend;
use rtrb::Producer;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::MediaStreamConstraints;
use web_sys::{AudioContext, AudioWorkletNode, MediaStream};

//audio_context is our device in cpalm or the sound interface
//AudioWorkletNode is our link to the Audio Worklet sending samples to main thread
pub struct WasmAudioBackend {
    audio_context: Option<AudioContext>,
    worklet_node: Option<AudioWorkletNode>,
    is_running: bool,
}

// https://developer.mozilla.org/fr/docs/Web/API/AudioWorklet

impl WasmAudioBackend {
    pub async fn new(producer: Producer<f32>) -> Result<Self, String> {
        //end point for web audio : can fail if the browser block audio permissions
        let audio_context =
            AudioContext::new().map_err(|e| format!("Failed to create AudioContext: {:?}", e))?;
        //load our async custom audio worklet defined in my-processor.js
        let worklet = audio_context
            .audio_worklet()
            .map_err(|_| "AudioWorklet not supported")?;

        //our js file, the AudioWorklet is async, so we need a promise here
        let promise = worklet
            .add_module("my-processor.js")
            .map_err(|e| format!("Faled to add module: {:?}", e))?;

        //we ask browser to load and compile our js file
        //we await our promise to actually load the worklet, or display error
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| format!("Failed to load worklet: {:?}", e))?;

        //The worklet node, links rust and AudioWorklet JS
        let worklet_node = AudioWorkletNode::new(&audio_context, "my-processor")
            .map_err(|e| format!("Failed to create worklet node: {:?}", e))?;

        //This handler allows us to receive the Float32Array from AudioWorklet JS
        Self::setup_message_handler(&worklet_node, producer)?;

        //This allows us to ask permission to user through the browser, to access microphone
        let media_stream = Self::get_user_media().await?;

        //The microphone input transit through this source node "MediaStreamSource"
        let source_node = audio_context
            .create_media_stream_source(&media_stream)
            .map_err(|e| format!("Failed to create media stream source: {:?}", e))?;

        //Connecting our source node to the worklet node, establish this pipeline
        //mic -> stream -> AudioWorklet
        source_node
            .connect_with_audio_node(&worklet_node)
            .map_err(|e| format!("Failed to connect source to worklet: {:?}", e))?;

        //audio feedback
        //worklet_node.connect_with_audio_node(&udio_context.destination())
        //  .map_err(|e| format!("Failed to connect to destination: {:?}", e))?;

        Ok(Self {
            audio_context: Some(audio_context),
            worklet_node: Some(worklet_node),
            is_running: false,
        })
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getUserMedia
    // this fct asks microphone access to user
    async fn get_user_media() -> Result<MediaStream, String> {
        //window is the browser object
        let window = web_sys::window().ok_or("No window object")?;

        let navigator = window.navigator();
        let media_devices = navigator
            .media_devices()
            .map_err(|_| "MediaDevices not supported")?;

        //require localhost or HTTPS : we ask to browser things about audio, but it could ignore our wish.
        //I will experiment in different browsers and situations to establish if i can safely ask for specific
        //sample rate, or if i have to deal with what browsers gives me anyway
        let constraints = MediaStreamConstraints::new();
        constraints.set_audio(&JsValue::TRUE);

        //this triggers the ask to user to allow access to microphone
        let promise = media_devices
            .get_user_media_with_constraints(&constraints)
            .map_err(|e| format!("getUserMedia failed: {:?}", e))?;

        //we await here, that user choose to allow us or not, access to microphone
        let result = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| format!("Failed to get media stream: {:?}", e))?;

        //The return of the promise, is a JsValue, so we need to convert it in a Rust type :
        //MediaStream
        let media_stream: MediaStream = result
            .dyn_into()
            .map_err(|_| "Failed to cast MediaStream")?;

        Ok(media_stream)
    }

    ///Receive samples sent by AudioWorklet and push them in the ringbuf
    fn setup_message_handler(
        worklet_node: &AudioWorkletNode,
        mut producer: Producer<f32>,
    ) -> Result<(), String> {
        //We define a closure, wrap to tranform rust function into JS callback
        //Since Rust and JS has differents memory management, we must wrap our closure so JS can
        //call it
        //move allows our closure to capture producer by value, now, the closure has the ownership
        let closure = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            //We must translate in the other way : JS sent to Rust Float32Array
            if let Ok(array) = event.data().dyn_into::<js_sys::Float32Array>() {
                //so we turn this array into a Vec<f32>, with we can work on in rust
                let samples = array.to_vec();
                //now, our samples are ready to be pushed in the ringbuf
                for sample in samples {
                    if let Err(e) = producer.push(sample) {
                        web_sys::console::error_1(
                            &format!("Failed to push sample in ringbuff : {e}").into(),
                        );
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        //we want to execute this closure for each message received from
        //AudioWorklet
        //uncheck_ref makes this part unsafe. I didn't found any better solution
        worklet_node
            .port()
            .map_err(|_| "Failed to get port")?
            .set_onmessage(Some(closure.as_ref().unchecked_ref()));

        //We do not want rust to drop the closure here, as it would do without this forget();
        //if the closure was dropped, JS would crash
        closure.forget();
        //todo : on/off at least for native to free this closure properly
        Ok(())
    }
}

impl AudioBackend for WasmAudioBackend {
    //do i want to switch start in async, to handle here promise and node ?
    fn start(&mut self) -> Result<(), String> {
        if self.is_running {
            return Ok(());
        }
        if let (Some(ctx), Some(_node)) = (&self.audio_context, &self.worklet_node) {
            let _promise = ctx
                .resume()
                .map_err(|e| format!("Failed to resume context: {:?}", e));
            self.is_running = true;
        }
        return Ok(());
    }

    fn stop(&mut self) {
        if let Some(ctx) = &self.audio_context {
            let _ = ctx.suspend();
        }
    }
}

// let audio_constraints = js_sys::Object::new();
//
// js_sys::Reflect::set(&audio_constraints, &"sampleRate".into(), &48000.into())?;
// js_sys::Reflect::set(&audio_constraints, &"channelCount".into(), &1.into())?;
// js_sys::Reflect::set(&audio_constraints, &"echoCancellation".into(), &false.into())?;
//
// constraints.set_audio(&audio_constraints.into());
