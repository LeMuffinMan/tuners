
use rtrb::{RingBuffer, Consumer, Producer};

pub const SAMPLE_RATE: u32 = 44100;
pub const BUFFER_SIZE: usize = 4096;

pub trait AudioBackend {
    fn start(&mut self);
    fn stop(&mut self);
}

pub struct AudioBridge {
    pub consumer: Consumer<f32>,
}

impl AudioBridge {
    pub fn new(buffer_capacity: usize) -> (Self, Producer<f32>) {
        let (producer, consumer) = RingBuffer::new(buffer_capacity);
        (Self { consumer }, producer)
    }
}
