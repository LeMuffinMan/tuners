

#[derive(Debug)]
pub struct AudioRingBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_pos: usize,
    read_pos: usize,
    len: usize,
}

#[derive(Debug, Clone)]
pub struct DSPRingBuffer {
    pub buffer: Vec<DSPResult>,
    pub capacity: usize,
    pub write_pos: usize,
    pub len: usize,
}

#[derive(Debug, Clone)]
pub struct DSPResult {
    pub rms: f32,
    // pub pitch: f32,
    // pub spectrum: Vec<f32>,
}


impl DSPRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![
                DSPResult { rms: 0.0 } ;
                capacity
            ],
            capacity,
            write_pos: 0,
            len: 0,
        }
    }

    pub fn push_rms(&mut self, rms: f32) {
        self.buffer[self.write_pos] = DSPResult { rms };
        self.write_pos = (self.write_pos + 1) % self.capacity;

        if self.len < self.capacity {
            self.len += 1;
        }
    }

    pub fn last(&self) -> Option<&DSPResult> {
        if self.len == 0 {
            None
        } else {
            let last_pos = (self.write_pos + self.capacity - 1) % self.capacity;
            Some(&self.buffer[last_pos])
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl AudioRingBuffer {
    pub fn peek_block(&self, out: &mut [f32]) -> usize {
        let n = out.len().min(self.len);
        let mut pos = self.read_pos;
        for i in 0..n {
            out[i] = self.buffer[pos];
            pos = (pos + 1) % self.capacity;
        }
        n
    }
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            write_pos: 0,
            read_pos: 0,
            len: 0,
        }
    }

    pub fn push_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;

            if self.len < self.capacity {
                self.len += 1;
            } else {
                self.read_pos = (self.read_pos + 1) % self.capacity;
            }
        }
    }

    pub fn pop_block(&mut self, out: &mut [f32]) -> usize {
        let n = out.len().min(self.len);
        for i in 0..n {
            out[i] = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
        self.len -= n;
        n
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

