use crate::ring::AudioRingBuffer;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
///This gives us a mutable global. This buffer is filled by audio side, and read by ui
///Rc and RefCell let us use the buffer as writer and reader in the same WASM thread, 
///checking at runtime to borrow it as mutable. 
///Panic in case of error
    pub static GLOBAL_RING: RefCell<Option<Rc<RefCell<AudioRingBuffer>>>> = RefCell::new(None);
}
