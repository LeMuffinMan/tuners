
/// Ring buffer pour stocker des samples audio
#[derive(Debug)]
pub struct AudioRingBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_pos: usize,
    read_pos: usize,
    len: usize,
}

impl AudioRingBuffer {
    /// Crée un nouveau ring buffer avec une capacité donnée
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            write_pos: 0,
            read_pos: 0,
            len: 0,
        }
    }

    /// Ajoute un bloc de samples au ring buffer
    pub fn push_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;

            if self.len < self.capacity {
                self.len += 1;
            } else {
                // On écrase l'ancien sample si full
                self.read_pos = (self.read_pos + 1) % self.capacity;
            }
        }
    }

    /// Lit un bloc de samples dans `out` et retourne le nombre lu
    pub fn pop_block(&mut self, out: &mut [f32]) -> usize {
        let n = out.len().min(self.len);
        for i in 0..n {
            out[i] = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
        self.len -= n;
        n
    }

    /// Nombre de samples actuellement dans le buffer
    pub fn len(&self) -> usize {
        self.len
    }

    /// Est-ce que le buffer est vide ?
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Capacité maximale
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

