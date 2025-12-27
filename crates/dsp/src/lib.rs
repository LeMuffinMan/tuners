
use rtrb::Consumer;
use audio::audio_bridge::BUFFER_SIZE;

pub struct DigitalSignalProcessor {
    consumer: Consumer<f32>,
    pub rms: f32,
    sample_buffer: Vec<f32>,
}

impl DigitalSignalProcessor {
    pub fn new(consumer: Consumer<f32>) -> Self {
        Self {
            consumer,
            sample_buffer: Vec::with_capacity(BUFFER_SIZE),
            rms: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.sample_buffer.clear();

        let mut count = 0;
        while let Ok(sample) = self.consumer.pop() {
            self.sample_buffer.push(sample);
            count += 1;
            if self.sample_buffer.len() >= BUFFER_SIZE {
                break;
            }
        }

        //faire une macro pour les logs pour egui / cli / wasm 
        if count > 0 {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("Read {} samples from ringbuffer", count).into());
        }

        if self.sample_buffer.is_empty() {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&"No samples available".into());
            self.rms = 0.0; 
            return; 
        };

        let sum: f32 = self.sample_buffer.iter().map(|&s| s * s).sum();
        self.rms = (sum / self.sample_buffer.len() as f32).sqrt();

        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("RMS calculated: {}", self.rms).into());
    }

    pub fn get_rms(&self) -> f32 {
        self.rms
    }
}



// pub fn freq_to_tune(freq: f32) -> String {
//         let tunes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
//         let a4 = 440.0;
//
//         let t = (12.0 * (freq / a4).log2()).round() as i32;
//         let tune_index = (t + 9).rem_euclid(12);
//         let octave = 4 + ((t + 9) / 12);
//
//         format!("{}{}", tunes[tune_index as usize], octave)
// }
//
// pub fn autocorrelation(buffer: &[f32], sample_rate: f32) -> Option<f32> {
//         let min_freq = 80.0;
//         let max_freq = 1000.0;
//
//         let min_lag = (sample_rate / max_freq) as usize;
//         let max_lag = (sample_rate / min_freq) as usize;
//
//         let mut best_lag = 0;
//         let mut best_corr = 0.0;
//
//         for lag in min_lag..max_lag {
//             let mut corr = 0.0;
//
//             for i in 0..(buffer.len() - lag) {
//                 corr += buffer[i] * buffer[i + lag];
//             }
//             if corr > best_corr {
//                 best_corr = corr;
//                 best_lag = lag;
//             }
//         }
//         if best_lag > 0 {
//             Some(sample_rate / best_lag as f32)
//         } else {
//             None
//         }
//     }
