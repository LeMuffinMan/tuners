use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use cpal::traits::DeviceTrait;
use audio::backend::native::start_audio_native;

use gui::{UiType, TunerApp};

use tuner_dsp::{autocorrelation, freq_to_tune};

//compile from tuners/ with cargo run -p tuners_native_gui

fn main() {
    start_audio_native();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Tuner WASM",
        options,
        Box::new(|_cc| Ok(Box::new(TunerApp::new(UiType::Desktop)))),
    );

}
