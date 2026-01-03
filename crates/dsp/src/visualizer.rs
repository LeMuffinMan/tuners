use clap::ValueEnum;

//rename to features
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum Visualizer {
    Freq,
    RMS,
    WaveForm,
}
