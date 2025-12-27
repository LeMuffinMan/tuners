
use gui::{UiType, TunerApp};
// use tuner_dsp::{autocorrelation, freq_to_tune};

//compile from tuners/ with cargo run -p tuners_native_gui

fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Tuner",
        options,
        Box::new(|_cc| Ok(Box::new(TunerApp::new(UiType::Desktop)))),
    );

}
