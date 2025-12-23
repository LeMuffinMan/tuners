use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use cpal::traits::DeviceTrait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("no input device available");
    let config = device.default_input_config().expect("no default config");

    // We NEVER want to interrupt the call back, so we need a shared buffer with a mutex to allow
    // another thread to operate while the main one is busy with continuous audio input callback

    //with capacity prevent reallocations 
    let shared_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(4096)));
    let buffer_for_audio = Arc::clone(&shared_buffer);
    let device_name = device.name().unwrap_or_else(|_| "unknown device".to_string());

    println!("Using device '{}' with config: {:?}", device_name, config);

    let input_stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // println!("received {} samples", data.len());
            let mut buf = buffer_for_audio.lock().expect("error locking buffer_for_audio mutex");

            //since data is a slice [], we can't push &[T] in buffer, we need extend_from_slice 
            buf.extend_from_slice(data); 
            let len = buf.len();
            if len > 4096 {
                let limit = len - 4096;
                //this delete the oldests samples to not grow in RAM
                buf.drain(0..limit);
            }
        },
        move |err| {
            println!("there was an error : {err}");
        },
        None, // Some(Duration) = timeout
    ).expect("failed to build input stream");

    input_stream.play().expect("failed to play stream");

    loop {
        std::thread::sleep(Duration::from_millis(100));

        let buffer = shared_buffer.lock().expect("error locking shared_buffer mutex");
        if buffer.is_empty() {
            continue;
        }

        //Root Mean Square : signal strength
        let rms = (buffer.iter().map(|x| x * x).sum::<f32>() / buffer.len() as f32).sqrt();

        // o2
        // for &s in buffer.iter().step_by(20) {
        //     let bar = (s.abs() * 100.0) as usize;
        //     println!("{: <50}", "#".repeat(bar));
        // }

        let bars = (rms * 100.0) as usize;
        println!("{: <50}", "█".repeat(bars));
    }
}
