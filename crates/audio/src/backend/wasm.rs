use web_sys::MediaStreamConstraints;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{AudioContext, AudioWorkletNode, MediaStream};
use rtrb::Producer;
use crate::backend::AudioBackend;

pub struct WasmAudioBackend {
    audio_context: Option<AudioContext>,
    worklet_node: Option<AudioWorkletNode>,
    is_running: bool,
}

//Since we depends of WebAudio API and AudioWorklet writing to this main thread the smples from the
//js my-processor.js, the process differs from native
impl WasmAudioBackend {
    pub async fn new(producer: Producer<f32>) -> Result<Self, String> {
        let audio_context = AudioContext::new()
            .map_err(|e| format!("Failed to create AudioContext: {:?}", e))?;
        let worklet = audio_context.audio_worklet()
            .map_err(|_| "AudioWorklet not supported")?;
        let promise = worklet.add_module("my-processor.js")
            .map_err(|e| format!("Faled to add module: {:?}", e))?;

        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| format!("Failed to load worklet: {:?}", e))?;

        let worklet_node = AudioWorkletNode::new(&audio_context, "my-processor")
            .map_err(|e| format!("Failed to create worklet node: {:?}", e))?;

        Self::setup_message_handler(&worklet_node, producer)?;

        let media_stream = Self::get_user_media().await?;

        let source_node = audio_context
            .create_media_stream_source(&media_stream)
            .map_err(|e| format!("Failed to create media stream source: {:?}", e))?;

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
    async fn get_user_media() -> Result<MediaStream, String> {
        let window = web_sys::window()
            .ok_or("No window object")?;

        let navigator = window.navigator();
        let media_devices = navigator.media_devices()
            .map_err(|_| "MediaDevices not supported")?;

        let constraints = MediaStreamConstraints::new();
        constraints.set_audio(&JsValue::TRUE);

        let promise = media_devices
            .get_user_media_with_constraints(&constraints)
            .map_err(|e| format!("getUserMedia failed: {:?}", e))?;

        let result = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| format!("Failed to get media stream: {:?}", e))?;

        let media_stream: MediaStream = result
            .dyn_into()
            .map_err(|_| "Failed to cast MediaStream")?;

        Ok(media_stream)
    }

    fn setup_message_handler(
        worklet_node: &AudioWorkletNode,
        mut producer: Producer<f32>,
    ) -> Result<(), String> {
        let closure = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            if let Ok(array) = event.data().dyn_into::<js_sys::Float32Array>() {
                let samples = array.to_vec();
                for sample in samples {
                    let _ = producer.push(sample);
                }
            }
        }) as Box<dyn FnMut(_)>);

        worklet_node.port()
            .map_err(|_| "Failed to get port")?
            .set_onmessage(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        Ok(())
    }
}

impl AudioBackend for WasmAudioBackend {
    fn start(&mut self) -> Result<(), String> {
        if self.is_running {
            return Ok(());
        }
        //faire qqchose de node ? promise ? 
        if let (Some(ctx), Some(_node)) = (&self.audio_context, &self.worklet_node) {
            let _promise = ctx.resume()
                .map_err(|e| format!("Failed to resume context: {:?}", e)) ;
            self.is_running = true;
        }
        return Ok(())
    }

    fn stop(&mut self) {
        if let Some(ctx) = &self.audio_context {
            let _ = ctx.suspend();
        }
    }
}

