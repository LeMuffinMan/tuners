use audio::audio_bridge::BUFFER_SIZE;
use rtrb::Consumer;
pub mod visualizer;
pub use visualizer::Visualizer;

/// Musical range we look for: the autocorrelation only scans the lags matching
/// these frequencies.
const MIN_FREQ: f32 = 80.0;
const MAX_FREQ: f32 = 1000.0;
/// Reference pitch of the 12 tone equal temperament we tune against.
const A4: f32 = 440.0;
/// How many samples we keep to analyse : 4096 at 48 kHz is 85 ms, enough for
/// seven periods of the lowest note we look for.
const WINDOW_SIZE: usize = 4096;
/// Below that energy we consider there is nothing to tune, and stop reporting a
/// note : the autocorrelation would otherwise find a pitch in the room noise.
const SILENCE_RMS: f32 = 0.01;

///We use this struct to compute on samples and store results ready to be displayed by ui
pub struct DigitalSignalProcessor {
    consumer: Consumer<f32>,
    pub rms: f32,
    sample_buffer: Vec<f32>,
    pub frequency: Option<f32>,
    pub note: Option<String>,
    pub cents: Option<f32>,
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
            cents: None,
            sample_rate: 48000.0,
        }
    }

    //we call this function in the eframe loop
    //at each frame, we update our sample_buffer so we work on the latests samples
    pub fn update(&mut self, feature: Visualizer) {
        let mut count = 0;
        while let Ok(sample) = self.consumer.pop() {
            self.sample_buffer.push(sample);
            count += 1;
        }
        //a frame only brings what arrived since the previous one, far less than
        //the autocorrelation needs : we slide a window instead of starting over
        if self.sample_buffer.len() > WINDOW_SIZE {
            let extra = self.sample_buffer.len() - WINDOW_SIZE;
            self.sample_buffer.drain(..extra);
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
            self.cents = None;
            return;
        };
        //For now we only calculare RMS, but data to display by ui will compute here
        let sum: f32 = self.sample_buffer.iter().map(|&s| s * s).sum();
        self.rms = (sum / self.sample_buffer.len() as f32).sqrt();

        if feature == Visualizer::Freq {
            let detected = if self.rms < SILENCE_RMS {
                None
            } else {
                Self::autocorrelation(&self.sample_buffer, self.sample_rate)
            };
            self.frequency = detected;
            self.note = detected.map(Self::freq_to_note);
            self.cents = detected.map(Self::freq_to_cents);
            #[cfg(target_arch = "wasm32")]
            if let Some(freq) = detected {
                web_sys::console::log_1(
                    &format!("Detected: {} Hz ({})", freq, Self::freq_to_note(freq)).into(),
                );
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

    pub fn get_cents(&self) -> Option<f32> {
        self.cents
    }

    //Position of a frequency on the 12 tone scale, 69.0 being A4 : the integer
    //part is the semitone, the fractional part is how far we are from it
    fn note_number(freq: f32) -> f32 {
        69.0 + 12.0 * (freq / A4).log2()
    }

    //Distance to the nearest semitone, in hundredth of a semitone
    fn freq_to_cents(freq: f32) -> f32 {
        let note_number = Self::note_number(freq);
        (note_number - note_number.round()) * 100.0
    }

    fn freq_to_note(freq: f32) -> String {
        let notes = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];

        let note_number = Self::note_number(freq).round() as i32;

        let note_index = note_number.rem_euclid(12);
        let octave = note_number / 12 - 1;

        format!("{}{}", notes[note_index as usize], octave)
    }

    fn autocorrelation(buffer: &[f32], sample_rate: f32) -> Option<f32> {
        let size = buffer.len();
        if size < 1024 {
            return None;
        }
        let min_lag = (sample_rate / MAX_FREQ) as usize;
        let max_lag = ((sample_rate / MIN_FREQ) as usize).min(size - 2);
        if min_lag < 2 || min_lag >= max_lag {
            return None;
        }

        let mean = buffer.iter().sum::<f32>() / size as f32;
        let signal: Vec<f32> = buffer.iter().map(|sample| sample - mean).collect();

        //one more lag on each side of the range: the peak interpolation needs them
        let mut corr = vec![0.0; max_lag + 2];
        for (lag, c) in corr.iter_mut().enumerate().skip(min_lag - 1) {
            *c = signal[..size - lag]
                .iter()
                .zip(&signal[lag..])
                .map(|(a, b)| a * b)
                .sum();
        }

        let mut best = min_lag;
        for (lag, c) in corr.iter().enumerate().take(max_lag + 1).skip(min_lag) {
            if *c > corr[best] {
                best = lag;
            }
        }
        if corr[best] <= 0.0 {
            return None;
        }

        //every multiple of the period correlates as well as the period itself :
        //taking the first lag that gets close to the best one keeps us from
        //hearing the note one or two octaves too low
        let threshold = corr[best] * 0.9;
        let peak = (min_lag..=max_lag)
            .find(|&lag| {
                corr[lag] >= threshold && corr[lag] >= corr[lag - 1] && corr[lag] >= corr[lag + 1]
            })
            .unwrap_or(best);
        Some(sample_rate / Self::interpolate_peak(&corr, peak))
    }

    //The peak lands between two samples : fitting a parabola on the peak and its
    //two neighbours gives the sub sample lag, so the frequency is not quantized
    //by the sample rate anymore
    fn interpolate_peak(corr: &[f32], peak: usize) -> f32 {
        let left = corr[peak - 1];
        let center = corr[peak];
        let right = corr[peak + 1];
        let denominator = left - 2.0 * center + right;
        if denominator == 0.0 {
            return peak as f32;
        }
        //a flat peak makes the fit unstable, and the real peak can never be more
        //than half a sample away from the one we found
        let offset = (0.5 * (left - right) / denominator).clamp(-0.5, 0.5);
        peak as f32 + offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48000.0;

    fn sine(freq: f32, len: usize) -> Vec<f32> {
        (0..len)
            .map(|i| (std::f32::consts::TAU * freq * i as f32 / SAMPLE_RATE).sin())
            .collect()
    }

    //same window as the one the processor slides in production
    fn detect(freq: f32) -> f32 {
        DigitalSignalProcessor::autocorrelation(&sine(freq, WINDOW_SIZE), SAMPLE_RATE)
            .expect("a sine inside the musical range must be detected")
    }

    #[test]
    fn detects_a440_within_one_cent() {
        let detected = detect(440.0);
        let error = DigitalSignalProcessor::freq_to_cents(detected).abs();
        assert!(error < 1.0, "detected {detected} Hz, {error} cents off");
    }

    #[test]
    fn detects_the_whole_musical_range() {
        for freq in [82.41, 110.0, 220.0, 587.33, 880.0] {
            let detected = detect(freq);
            let cents = 1200.0 * (detected / freq).log2();
            assert!(cents.abs() < 5.0, "{freq} Hz detected as {detected} Hz");
        }
    }

    #[test]
    fn names_the_note_and_its_deviation() {
        assert_eq!(DigitalSignalProcessor::freq_to_note(440.0), "A4");
        assert_eq!(DigitalSignalProcessor::freq_to_note(261.63), "C4");
        assert!(DigitalSignalProcessor::freq_to_cents(440.0).abs() < 0.01);

        //a quarter tone above A4 sits exactly between two semitones
        let quarter_tone = 440.0 * 2.0_f32.powf(0.5 / 12.0);
        let cents = DigitalSignalProcessor::freq_to_cents(quarter_tone);
        assert!((cents.abs() - 50.0).abs() < 0.5, "{cents} cents");
    }

    #[test]
    fn silence_is_not_a_note() {
        let silence = vec![0.0; WINDOW_SIZE];
        assert!(DigitalSignalProcessor::autocorrelation(&silence, SAMPLE_RATE).is_none());
    }
}
