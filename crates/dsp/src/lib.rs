use audio::audio_bridge::{BUFFER_SIZE, SAMPLE_RATE};
use rtrb::Consumer;

///We use this struct to compute on samples and store results ready to be displayed by ui
pub struct DigitalSignalProcessor {
    consumer: Consumer<f32>,
    pub rms: f32,
    sample_buffer: Vec<f32>,
    pub frequency: Option<f32>,
    pub note: Option<String>,
    sample_rate: f32,
}

//The Audio Callback async rust function or the AudioWorklet will write samples in the ring buf
//the consumer end allows us to read it
impl DigitalSignalProcessor {
    pub fn new(consumer: Consumer<f32>) -> Self {
        Self {
            consumer,
            sample_buffer: Vec::with_capacity(BUFFER_SIZE),
            rms: 0.0,
            frequency: None,
            note: None,
            sample_rate: SAMPLE_RATE as f32,
        }
    }

    //we call this function in the eframe loop
    //at each frame, we update our sample_buffer so we work on the latests samples
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
            self.frequency = None;
            self.note = None;
            return;
        };
        //For now we only calculare RMS, but data to display by ui will compute here
        let sum: f32 = self.sample_buffer.iter().map(|&s| s * s).sum();
        self.rms = (sum / self.sample_buffer.len() as f32).sqrt();
        
        if let Some(freq) = self.autocorrelation(&self.sample_buffer, self.sample_rate) {
            self.frequency = Some(freq);
            self.note = Some(Self::freq_to_note(freq));
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("Detected: {} Hz ({})", freq, Self::freq_to_note(freq)).into());
        } else {
            self.frequency = None;
            self.note = None;
        }
        
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("RMS: {}", self.rms).into());
    }

    pub fn get_rms(&self) -> f32 {
        self.rms
    }
    pub fn get_samples(&self, count: usize) -> Vec<f32> {
        if self.sample_buffer.is_empty() {
            return Vec::new();
        }
        
        let buffer_len = self.sample_buffer.len();
        
        if count >= buffer_len {
            return self.sample_buffer.clone();
        }
        
        let step = buffer_len as f32 / count as f32;
        let mut samples = Vec::with_capacity(count);
        
        for i in 0..count {
            let index = (i as f32 * step) as usize;
            if index < buffer_len {
                samples.push(self.sample_buffer[index]);
            }
        }
        
        samples
    }
    pub fn get_frequency(&self) -> Option<f32> {
        self.frequency
    }
    
    pub fn get_note(&self) -> Option<String> {
        self.note.clone()
    }
    
    fn freq_to_note(freq: f32) -> String {
        let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let a4 = 440.0;
        
        let half_steps = (12.0 * (freq / a4).log2()).round() as i32;
        let note_index = (half_steps + 9).rem_euclid(12);
        let octave = 4 + ((half_steps + 9) / 12);
        
        format!("{}{}", notes[note_index as usize], octave)
    }
    
    fn autocorrelation(&self, buffer: &[f32], sample_rate: f32) -> Option<f32> {
        if buffer.len() < 4096 {
            return None;
        }
        
        let rms = self.rms;
        // if rms < 0.005 {
        //     return None;
        // }
        
        //mode guitare / basse / ...
        let min_freq = 35.0;   
        let max_freq = 400.0;  
        
        let min_lag = (sample_rate / max_freq) as usize;
        let max_lag = (sample_rate / min_freq).min(buffer.len() as f32 / 2.0) as usize;

        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("Buffer len: {}, min_lag: {}, max_lag: {}, RMS: {}", 
            buffer.len(), min_lag, max_lag, rms).into());
        
        let mean: f32 = buffer.iter().sum::<f32>() / buffer.len() as f32;
        let normalized: Vec<f32> = buffer.iter().map(|&s| s - mean).collect();
        
        let mut best_lag = 0;
        let mut best_corr = -1.0;
        let mut second_best_corr = -1.0;
        
        for lag in min_lag..max_lag {
            let mut corr = 0.0;
            let mut norm1 = 0.0;
            let mut norm2 = 0.0;
            
            for i in 0..(normalized.len() - lag) {
                corr += normalized[i] * normalized[i + lag];
                norm1 += normalized[i] * normalized[i];
                norm2 += normalized[i + lag] * normalized[i + lag];
            }
            
            if norm1 > 0.0 && norm2 > 0.0 {
                corr /= (norm1 * norm2).sqrt();
            }
            
            if corr > best_corr {
                second_best_corr = best_corr;
                best_corr = corr;
                best_lag = lag;
            } else if corr > second_best_corr {
                second_best_corr = corr;
            }
        }
        
        let clarity = best_corr - second_best_corr;
        
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("Best corr: {}, best_lag: {}", best_corr, best_lag).into());
   

        if best_lag > 0 && best_corr > 0.3 && clarity > 0.1 {
            let refined_lag = if best_lag > min_lag && best_lag < max_lag - 1 {
                self.parabolic_interpolation(&normalized, best_lag)
            } else {
                best_lag as f32
            };
            let freq = sample_rate / refined_lag;
            
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("Detected frequency: {} Hz", freq).into());
            
            Some(freq)
        } else {
            None
        }
    }
    
    fn parabolic_interpolation(&self, buffer: &[f32], lag: usize) -> f32 {
        if lag == 0 || lag >= buffer.len() - 1 {
            return lag as f32;
        }
        
        let alpha = self.compute_correlation(buffer, lag - 1);
        let beta = self.compute_correlation(buffer, lag);
        let gamma = self.compute_correlation(buffer, lag + 1);
        
        let denom = alpha - 2.0 * beta + gamma;
        if denom.abs() < 0.0001 {
            return lag as f32;
        }
        
        let offset = 0.5 * (alpha - gamma) / denom;
        
        let offset = offset.clamp(-1.0, 1.0);
        
        lag as f32 + offset
    }
    
    fn compute_correlation(&self, buffer: &[f32], lag: usize) -> f32 {
        if lag >= buffer.len() {
            return 0.0;
        }
        
        let mut corr = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;
        
        for i in 0..(buffer.len() - lag) {
            corr += buffer[i] * buffer[i + lag];
            norm1 += buffer[i] * buffer[i];
            norm2 += buffer[i + lag] * buffer[i + lag];
        }
        
        if norm1 > 0.0 && norm2 > 0.0 {
            corr / (norm1 * norm2).sqrt()
        } else {
            0.0
        }
    }}
