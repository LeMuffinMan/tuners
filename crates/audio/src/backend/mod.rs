
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(target_arch = "wasm32")]
pub use wasm::start_audio;

// #[cfg(not(target_arch = "wasm32"))]
// pub use native::start_audio;
