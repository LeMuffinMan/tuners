pub mod audio_bridge;
pub mod backend;

#[cfg(not(target_arch = "wasm32"))]
pub use backend::native::NativeAudioBackend;
#[cfg(target_arch = "wasm32")]
pub use backend::wasm::WasmAudioBackend;
