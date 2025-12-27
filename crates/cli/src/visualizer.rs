use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Visualizer {
    Freq,
    RMS,
    WaveShape,
}
