
use super::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;

pub struct NativeBackend {
    stream: Option<Stream>,
}

impl NativeBackend {
    pub new(mut producer, Producer<f32>) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("No input device")?;
        let config = device.default_input_config()?;

        let producer = self.ring.producer();

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for &sample in data {
                    let _ = producer.push(sample);
                }
            },
            move |err| eprintln!("CPAL input error: {:?}", err),
            None,
        )?;

        Ok(Self {
            stream: Some(stream),
        })
    }
}

impl AudioBackend for NativeBackend {
    fn start(&mut self) -> Result<(), String> {
        if let Some(stream) = &self.stream {
            stream.play().map_err(|e| format!("Failed to play stream : {e}"))?;
        }
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(stream) = &self.stream {
            let _ = stream.pause();
        }
    }
}
