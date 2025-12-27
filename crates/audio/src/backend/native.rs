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
        
        let stream = device
            .build_input_stream(
                &config.into(), // config est un SupportedStreamConfig, .into() le convertit
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    for &sample in data {
                        let _ = producer.push(sample);
                    }
                },
                move |err| eprintln!("CPAL input error: {:?}", err),
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
