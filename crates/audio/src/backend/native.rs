use crate::ring::AudioRingBuffer;
use std::cell::RefCell;
use std::rc::Rc;
use cpal::traits::HostTrait;
use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;

thread_local! {
///This gives us a mutable global. This buffer is filled by audio side, and read by ui
///Rc and RefCell let us use the buffer as writer and reader in the same WASM thread, 
///checking at runtime to borrow it as mutable. 
///Panic in case of error
    pub static GLOBAL_RING: RefCell<Option<Rc<RefCell<AudioRingBuffer>>>> = RefCell::new(None);
}

pub fn start_audio_native() {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("no input device available");

    let config = device.default_input_config().expect("no default config");
    let sample_rate = config.sample_rate();
    let capacity = 4096;
    GLOBAL_RING.with(|g| {
        *g.borrow_mut() = Some(Rc::new(RefCell::new(AudioRingBuffer::new(capacity))));
    });
    let input_stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            GLOBAL_RING.with(|g| {
                if let Some(ring_rc) = &*g.borrow() {
                    let mut ring = ring_rc.borrow_mut();
                    ring.push_slice(data);
                }
            });
        },
        move |err| {
            println!("there was an error: {err}");
        },
        None, // timeout
    ).expect("failed to build input stream");
    input_stream.play().expect("failed to play stream");
}
