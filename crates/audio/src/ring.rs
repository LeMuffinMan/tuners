
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
        let (mut producer, mut consumer) = RingBuffer::new(buffer_capacity);
        (Self { consumer }, producer)
    }
}


//
//
//
// use rtrb::{RingBuffer, Producer, Consumer};
//
// pub struct AudioRingBuffer {
//     producer: Producer<f32>,
//     consumer: Consumer<f32>,
//     buffer: Vec<f32>, // buffer temporaire pour UI
// }
//
// impl AudioRingBuffer {
//     pub fn new(size: usize) -> Self {
//         let (producer, consumer) = RingBuffer::<f32>::new(size);
//         Self { producer, consumer, buffer: Vec::new() }
//     }
//
//     pub fn push(&mut self, sample: f32) {
//         let _ = self.producer.push(sample);
//     }
//
//     pub fn read_all(&mut self) -> &[f32] {
//         self.buffer.clear();
//         while let Ok(s) = self.consumer.pop() {
//             self.buffer.push(s);
//         }
//         &self.buffer
//     }
//
//     pub fn rms(&self) -> f32 {
//         if self.buffer.is_empty() { return 0.0; }
//         let sum: f32 = self.buffer.iter().map(|x| x*x).sum();
//         (sum / self.buffer.len() as f32).sqrt()
//     }
//
//     pub fn producer(&mut self) -> &mut Producer<f32> {
//         &mut self.producer
//     }
//
//     pub fn consumer(&mut self) -> &mut Consumer<f32> {
//         &mut self.consumer
//     }
// }
//
