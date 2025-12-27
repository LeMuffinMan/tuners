use rtrb::{Consumer, Producer, RingBuffer};

//are we sure we can ask browser our sample rate with guarantee or we use default setting ?
pub const SAMPLE_RATE: u32 = 44100;
pub const BUFFER_SIZE: usize = 4096;

pub struct AudioBridge {
    pub consumer: Consumer<f32>,
}

//AudioBridge is our ringbuff : at creation, we will want to plug the producer in the AudioWorklet
//or in our audio callback with cpal. the consumer end will be used by the DSP to compute and
//provide final data to Ui
impl AudioBridge {
    pub fn new(buffer_capacity: usize) -> (Self, Producer<f32>) {
        let (producer, consumer) = RingBuffer::new(buffer_capacity);
        (Self { consumer }, producer)
    }
}
