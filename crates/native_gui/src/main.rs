use audio::ring::{ AudioBridge, SAMPLE_RATE };
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
                std::thread::sleep(Duration::from_millis(100));
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
