use audio::audio_bridge::{ AudioBridge, SAMPLE_RATE };
use gui::ui::DigitalSignalProcessor;
use audio::NativeAudioBackend;
use gui::{UiType, TunerApp, Visualizer};
use clap::{ Parser, ValueEnum };
use audio::backend::AudioBackend;
use std::time::Duration;

//compile with cargo run -p tuners_native_gui

//renommer UiType
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Ui {
    Gui,
    Cli,
    // Tui, 
}

#[derive(Parser)]
#[command(name = "Tuners")]
#[command(about = "A simple tuner and sound visualizer")]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Select the Ui to launch", value_enum, default_value_t = Ui::Gui)]
    ui: Ui,
    #[arg(short, long, help = "Select feature", value_enum, default_value_t = Visualizer::RMS)]
    visualizer: Visualizer, 
}

fn main() {
    let args = Args::parse();
    match args.ui {
        Ui::Gui => {
            let options = eframe::NativeOptions::default();
            let _ = eframe::run_native(
                "Tuner",
                options,
                Box::new(|_cc| Ok(Box::new(TunerApp::new(UiType::Desktop)))),
            );
        },
        Ui::Cli => {

            let (bridge, producer) = AudioBridge::new(SAMPLE_RATE as usize * 2);
            let mut dsp = DigitalSignalProcessor::new(bridge.consumer);
            let mut backend = match NativeAudioBackend::new(producer) {
                Ok(backend) =>  backend,
                Err(e) => {
                    eprintln!("Failed to create audio backend: {}", e);
                    return;
                }
            };

            if let Err(e) = backend.start() {
                eprintln!("Failed to start backend: {}", e);
            }

            loop {
                std::thread::sleep(Duration::from_millis(8));
                dsp.update();
                match args.visualizer {
                    Visualizer::RMS => {
                        let bars = (dsp.get_rms() * 50.0) as usize;
                        println!("{: <50}", "█".repeat(bars));
                    },
                    Visualizer::WaveShape => {

                    },
                    Visualizer::Freq => {

                    },
                }
            }
        },
        // Ui::Tui => {
        //
        // }        
    }

}

        // Wave shape
        // for &s in buffer.iter().step_by(20) {
        //     let bar = (s.abs() * 100.0) as usize;
        //     println!("{: <50}", "#".repeat(bar));
        // }

        // if buffer.len() < 2048 {
        //     continue;
        // }

        // if rms > 0.1 {
        //     if let Some(freq) = autocorrelation(&buffer, sample_rate as f32) {
        //         let tune = freq_to_tune(freq); 
        //         println!("Freq: {:.2}Hz | {} | rms = {}", freq, tune, rms);
        //     }
        // }
    //
    //
    // fn freq_to_tune(freq: f32) -> String {
    //     let tunes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    //     let a4 = 440.0;
    //
    //     let t = (12.0 * (freq / a4).log2()).round() as i32;
    //     let tune_index = (t + 9).rem_euclid(12);
    //     let octave = 4 + ((t + 9) / 12);
    //
    //     format!("{}{}", tunes[tune_index as usize], octave)
    // }
    //
    // fn autocorrelation(buffer: &[f32], sample_rate: f32) -> Option<f32> {
    //     let min_freq = 80.0;
    //     let max_freq = 1000.0;
    //
    //     let min_lag = (sample_rate / max_freq) as usize;
    //     let max_lag = (sample_rate / min_freq) as usize;
    //
    //     let mut best_lag = 0;
    //     let mut best_corr = 0.0;
    //
    //     for lag in min_lag..max_lag {
    //         let mut corr = 0.0;
    //
    //         for i in 0..(buffer.len() - lag) {
    //             corr += buffer[i] * buffer[i + lag];
    //         }
    //         if corr > best_corr {
    //             best_corr = corr;
    //             best_lag = lag;
    //         }
    //     }
    //     if best_lag > 0 {
    //         Some(sample_rate / best_lag as f32)
    //     } else {
    //         None
    //     }
    // }
