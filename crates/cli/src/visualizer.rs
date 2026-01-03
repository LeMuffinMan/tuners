use clap::ValueEnum;

//rename to features
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Visualizer {
    Freq,
    RMS,
    WaveShape,
}
