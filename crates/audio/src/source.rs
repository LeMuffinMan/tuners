
pub trait AudioSource {
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> usize;
    fn start(&mut self);
    fn stop(&mut self);
}
