use crate::ring::{AudioRingBuffer, DSPRingBuffer};

pub trait RingReader {
    fn get_rms(&mut self) -> f32;
}

//Native 
#[cfg(not(target_arch = "wasm32"))]
impl RingReader for AudioRingBuffer {
    fn get_rms(&mut self) -> f32 {
        0.0
    }
}

//Wasm
#[cfg(target_arch = "wasm32")]
impl RingReader for DSPRingBuffer {
    fn get_rms(&mut self) -> f32 {
        if self.len == 0 { return 0.0; }
        let last_pos = (self.write_pos + self.capacity - 1) % self.capacity;
        self.buffer[last_pos].rms
    }
}

