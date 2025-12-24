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

    let sample_rate = config.sample_rate();
    // We NEVER want to interrupt the call back, so we need a shared buffer with a mutex :
    //
    // - The Callback thread handle the callback and store the 4096 last samples in a sharebuff
    // - The 2nd thread compute with data from this shared buff  without interfering with the continuous callback

    let shared_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(4096)));
    let buffer_for_audio = Arc::clone(&shared_buffer);
    let device_name = device.name().unwrap_or_else(|_| "unknown device".to_string());

    println!("Using device '{}' with config: {:?}", device_name, config);
    //The audio callback is a loop, it fills data variable with audio data from our input source continuously.
    //
    //data is a slice of f32 : each float is a sample, expressing an amplitude value at a specific
    //time, as a value from -1 to +1. ex : [-0.9873, 0.3523464, 0.2, ...]
    //
    //if the config of my default input device has a sampling rate of 48 000 Hz, it means i need 
    //48 000 of theses samples, to have 1 second of audio data available on my shared buffer
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

        // let bars = (rms * 100.0) as usize;
        // println!("{: <50}", "█".repeat(bars));

        //Wave shape
        // for &s in buffer.iter().step_by(20) {
        //     let bar = (s.abs() * 100.0) as usize;
        //     println!("{: <50}", "#".repeat(bar));
        // }

        if buffer.len() < 2048 {
            continue;
        }

        if rms > 0.1 {
            if let Some(freq) = autocorrelation(&buffer, sample_rate as f32) {
                let tune = freq_to_tune(freq); 
                println!("Freq: {:.2}Hz | {} | rms = {}", freq, tune, rms);
            }
        }
    }


    fn freq_to_tune(freq: f32) -> String {
        let tunes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let a4 = 440.0;

        let t = (12.0 * (freq / a4).log2()).round() as i32;
        let tune_index = (t + 9).rem_euclid(12);
        let octave = 4 + ((t + 9) / 12);

        format!("{}{}", tunes[tune_index as usize], octave)
    }

    fn autocorrelation(buffer: &[f32], sample_rate: f32) -> Option<f32> {
        let min_freq = 80.0;
        let max_freq = 1000.0;

        let min_lag = (sample_rate / max_freq) as usize;
        let max_lag = (sample_rate / min_freq) as usize;

        let mut best_lag = 0;
        let mut best_corr = 0.0;

        for lag in min_lag..max_lag {
            let mut corr = 0.0;

            for i in 0..(buffer.len() - lag) {
                corr += buffer[i] * buffer[i + lag];
            }
            if corr > best_corr {
                best_corr = corr;
                best_lag = lag;
            }
        }
        if best_lag > 0 {
            Some(sample_rate / best_lag as f32)
        } else {
            None
        }
    }
}
