pub fn freq_to_tune(freq: f32) -> String {
        let tunes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let a4 = 440.0;

        let t = (12.0 * (freq / a4).log2()).round() as i32;
        let tune_index = (t + 9).rem_euclid(12);
        let octave = 4 + ((t + 9) / 12);

        format!("{}{}", tunes[tune_index as usize], octave)
}

pub fn autocorrelation(buffer: &[f32], sample_rate: f32) -> Option<f32> {
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
