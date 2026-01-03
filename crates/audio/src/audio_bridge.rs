use rtrb::{Consumer, Producer, RingBuffer};

pub const BUFFER_SIZE: usize = 4096;

pub struct AudioBridge {
    pub consumer: Consumer<f32>,
    // pub sample_rate: f32,
}

//AudioBridge is our ringbuff : at creation, we will want to plug the producer in the AudioWorklet
//or in our audio callback with cpal. the consumer end will be used by the DSP to compute and
//provide final data to Ui
impl AudioBridge {
    pub fn new() -> (Self, Producer<f32>) {
        let (producer, consumer) = RingBuffer::new(96000);
        (Self { consumer }, producer)
    }
}
