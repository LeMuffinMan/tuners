use super::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use rtrb::Producer;

pub struct NativeAudioBackend {
    stream: Option<Stream>,
}

impl NativeAudioBackend {
    pub fn new(mut producer: Producer<f32>) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device")?;
        
        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get config: {}", e))?;
        
        //move on a closure force the closure to capture variables by value : here it captures
        //producer taking its ownership. Moving producer make it usable only inside this closure, 
        //where cpal runs our Audio Callback. CPAL wants it to stay alive, so we must give
        //ownership to its closure.
        //To not interfer with this audio callback and work in real time, we only push our sample
        //on the ringbuff and nothing else. The consumer will get samples through the ringbuff, DSP process it and UI renders it.
        //data is the samples themself, so we can iterate for each sample to push them in the
        //ringbuf
        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    for &sample in data {
                        let _ = producer.push(sample);
                    }
                },
                move |err| eprintln!("cpal input error: {:?}", err),
                None,
            )
            .map_err(|e| format!("Failed to build stream: {}", e))?;
        
        Ok(Self {
            stream: Some(stream),
        })
    }
}

impl AudioBackend for NativeAudioBackend {
    fn start(&mut self) -> Result<(), String> {
        if let Some(stream) = &self.stream {
            stream
                .play()
                .map_err(|e| format!("Failed to play stream: {}", e))?;
        }
        Ok(())
    }
    
    fn stop(&mut self) {
        if let Some(stream) = &self.stream {
            let _ = stream.pause();
        }
    }
}
