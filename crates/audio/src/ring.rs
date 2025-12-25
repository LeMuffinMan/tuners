
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
///This gives us a mutable global. This buffer is filled by audio side, and read by ui
///Rc and RefCell let us use the buffer as writer and reader in the same WASM thread, 
///checking at runtime to borrow it as mutable. 
///Panic in case of error
    pub static GLOBAL_RING: RefCell<Option<Rc<RefCell<AudioRingBuffer>>>> = RefCell::new(None);
}

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

