use audio::audio_bridge::BUFFER_SIZE;
use rtrb::Consumer;
pub mod visualizer;
pub use visualizer::Visualizer;

///We use this struct to compute on samples and store results ready to be displayed by ui
pub struct DigitalSignalProcessor {
    consumer: Consumer<f32>,
    pub rms: f32,
    sample_buffer: Vec<f32>,
    pub frequency: Option<f32>,
    pub note: Option<String>,
    pub sample_rate: f32,
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
            sample_rate: 48000.0,
        }
    }

    //we call this function in the eframe loop
    //at each frame, we update our sample_buffer so we work on the latests samples
    pub fn update(&mut self, feature: Visualizer) {
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

        if feature == Visualizer::Freq {
            if let Some(freq) = self.autocorrelation(&self.sample_buffer, self.sample_rate) {
                self.frequency = Some(freq);
                self.note = Some(Self::freq_to_note(freq));
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(
                    &format!("Detected: {} Hz ({})", freq, Self::freq_to_note(freq)).into(),
                );
            } else {
                self.frequency = None;
                self.note = None;
            }
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
        let notes = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];

        let a4 = 440.0;
        let note_number = 69.0 + 12.0 * (freq / a4).log2();
        let note_number = note_number.round() as i32;

        let note_index = note_number.rem_euclid(12);
        let octave = note_number / 12 - 1;

        format!("{}{}", notes[note_index as usize], octave)
    }

    fn autocorrelation(&self, buffer: &[f32], sample_rate: f32) -> Option<f32> {
        let size = buffer.len();
        if size < 1024 {
            return None;
        }

        let rms = (buffer.iter().map(|x| x * x).sum::<f32>() / size as f32).sqrt();
        // if rms < 0.01 {
        //     return None;
        // }

        let mean = buffer.iter().sum::<f32>() / size as f32;
        let mut signal = Vec::with_capacity(size);
        for &x in buffer {
            signal.push(x - mean);
        }

        let mut corr = vec![0.0; size];
        for lag in 0..size {
            let mut sum = 0.0;
            for i in 0..(size - lag) {
                sum += signal[i] * signal[i + lag];
            }
            corr[lag] = sum;
        }

        let mut d = 0;
        while d + 1 < size && corr[d] > corr[d + 1] {
            d += 1;
        }

        let mut max_pos = d;
        let mut max_val = corr[d];
        for (i, c) in corr.iter_mut().enumerate().take(size).skip(d) {
            if *c > max_val {
                max_val = *c;
                max_pos = i;
            }
        }

        if max_pos == 0 {
            return None;
        }

        Some(sample_rate / max_pos as f32)
    }
}
