pub mod source;
pub mod ring;
pub mod backend;

#[cfg(not(target_arch = "wasm32"))]
pub use backend::native::NativeAudioBackend;
#[cfg(target_arch = "wasm32")]
pub use backend::wasm::WasmAudioBackend;
// pub use ring::AudioRingBuffer;
// pub use ring_reader::RingReader;

#[cfg(test)]
mod tests {
    use super::engine::{AudioEngine, DspProcessor};
    use super::ring::AudioRingBuffer;

    pub struct MockDsp {
        last: f32,
    }

    impl DspProcessor for MockDsp {
        fn process_block(&mut self, block: &[f32]) {
            self.last = block.iter().sum::<f32>();
        }

        fn latest_result(&self) -> f32 {
            self.last
        }
    }

    #[test]
    fn test_engine_push() {
        let dsp = MockDsp { last: 0.0 };
        let mut engine = AudioEngine::new(dsp, 4096);

        let samples: Vec<f32> = (0..3000).map(|x| x as f32).collect();
        engine.push_samples(&samples);
        assert!(engine.latest_result() > 0.0);
    }
}
