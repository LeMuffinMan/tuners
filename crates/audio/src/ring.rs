
use rtrb::RingBuffer;

pub struct AudioRingBuffer {
    producer: rtrb::Producer<f32>,
    consumer: rtrb::Consumer<f32>,
}

impl AudioRingBuffer {
    pub fn new(capacity: usize) -> Self {
        let (producer, consumer) = RingBuffer::<f32>::new(capacity);
        Self { producer, consumer }
    }

    pub fn push_samples(&mut self, samples: &[f32]) {
        for &s in samples {
            let _ = self.producer.push(s); // Ok(()) ou Err(s) si plein
        }
    }

    pub fn pop_block(&mut self, block: &mut [f32]) -> usize {
        let mut count = 0;
        for slot in block.iter_mut() {
            match self.consumer.pop() {
                Ok(sample) => {
                    *slot = sample;
                    count += 1;
                }
                Err(_) => break,
            }
        }
        count
    }

    pub fn len(&self) -> usize {
        self.consumer.slots() // nombre d’éléments dispo
    }
}

