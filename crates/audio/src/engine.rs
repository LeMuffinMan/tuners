
use crate::ring::AudioRingBuffer;
use crate::block::{BLOCK_SIZE, HOP_SIZE};

pub trait DspProcessor {
    fn process_block(&mut self, block: &[f32]);
    fn latest_result(&self) -> f32;
}

pub struct AudioEngine<D: DspProcessor> {
    ring: AudioRingBuffer,
    dsp: D,
    block_size: usize,
}

impl<D: DspProcessor> AudioEngine<D> {
    pub fn new(dsp: D, ring_capacity: usize) -> Self {
        Self {
            ring: AudioRingBuffer::new(ring_capacity),
            dsp,
            block_size: BLOCK_SIZE,
        }
    }

    pub fn push_samples(&mut self, samples: &[f32]) {
        self.ring.push_samples(samples);

        while self.ring.len() >= self.block_size {
            let mut block = vec![0.0; self.block_size];
            self.ring.pop_block(&mut block);
            self.dsp.process_block(&block);
        }
    }

    pub fn latest_result(&self) -> f32 {
        self.dsp.latest_result()
    }
}
