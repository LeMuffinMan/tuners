
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(target_arch = "wasm32")]
pub use wasm::WasmAudioBackend;

pub trait AudioBackend {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    // fn output_ring(&self) -> &crate::ring::AudioRingBuffer;
}


// #[cfg(not(target_arch = "wasm32"))]
// pub use native::start_audio;
