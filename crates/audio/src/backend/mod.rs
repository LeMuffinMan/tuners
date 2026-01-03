#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::WasmAudioBackend;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

///This trait allows us to abstract the backend.
///using start and stop will call either the wasm or native one without duplicating code
pub trait AudioBackend {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    fn sample_rate(&self) -> f32;
}
