# Tune.rs

A real-time audio DSP engine in Rust. Captures live microphone input, extracts signal features (RMS, waveform, frequency detection), and visualizes them — on both native desktop and web (WASM).

→ [Try it live](https://lemuffinman.github.io/tuners)

---

## Features

- **Live audio capture** — microphone input via CPAL (native) or Web Audio API / AudioWorklet (WASM)
- **Signal processing** — RMS energy, waveform visualization, frequency detection with note name (autocorrelation, A4 = 440 Hz)
- **Responsive GUI** — desktop and mobile layouts (egui/eframe)
- **CLI mode** — headless RMS bar in terminal (`--ui cli`)
- **Dual compilation** — same core codebase, two targets: native binary and WASM

---

## Architecture

The central design goal: a single audio + DSP pipeline shared across targets, with all platform-specific code isolated at the edges.

```
crates/
├── audio/    — AudioBackend trait + NativeAudioBackend (CPAL) + WasmAudioBackend (Web Audio API)
├── dsp/      — DigitalSignalProcessor: RMS, waveform, autocorrelation, note detection
├── gui/      — egui UI: desktop layout + mobile layout
├── native/   — native binary entry point, clap CLI parsing
└── wasm/     — WASM entry point (wasm-bindgen, AudioWorklet bridge)
```

**Execution pipeline:**

```
Native:  CPAL callback → RingBuffer (rtrb) → DSP → GUI / CLI
WASM:    AudioWorklet (JS) → MessagePort → RingBuffer (rtrb) → DSP → GUI
```

---

## Key design decisions

### AudioBackend trait

Both targets implement the same interface:

```rust
trait AudioBackend {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    fn sample_rate(&self) -> f32;
}
```

- `NativeAudioBackend` — CPAL audio callback, `Producer<f32>` moved into the closure
- `WasmAudioBackend` — async init, AudioWorklet module loaded via promise, samples received through `MessagePort` → `Float32Array`

This keeps DSP and GUI completely platform-agnostic. `cfg` gates are confined to backend selection and a few initialization paths.

### Lock-free ring buffer

Audio callbacks are real-time contexts: no allocation, no blocking. The boundary between audio capture and the rest of the system is a single-producer / single-consumer lock-free ring buffer (`rtrb`, capacity 96 000 samples ≈ 2s at 48 kHz).

- **Producer** — owned by the audio callback, pushes raw `f32` samples
- **Consumer** — owned by the DSP, drained each frame

This works on both targets: CPAL on native, and on WASM where the AudioWorklet forwards samples via `MessagePort` (no `SharedArrayBuffer` available).

### DSP

Each frame, `DigitalSignalProcessor::update()` drains the ring buffer and computes:

| Feature | Method |
|---|---|
| RMS | `sqrt(sum(s²) / n)` |
| Waveform | Downsampled window of raw samples |
| Frequency | Autocorrelation on 1024+ sample buffer, peak lag → Hz |

### GUI

The GUI crate (egui/eframe) adapts its layout at runtime:

- **Desktop** — control panel (start/stop, visualizer selector) + side panel (source code link) + central visualization
- **Mobile** — larger fonts, stacked layout, touch-friendly controls

The same `TunerApp` struct handles both layouts.

---

## Build & Run

### Native

```bash
# GUI (default)
cargo run -p tuners_native

# CLI — RMS bar in terminal
cargo run -p tuners_native -- --ui cli --visualizer rms
```

Available visualizers: `rms`, `freq`, `wave-form`

### WASM

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd crates/wasm && trunk serve
```

---

## Resources

- [rtrb — lock-free ring buffer](https://github.com/mgeier/rtrb)
- [CPAL — cross-platform audio](https://github.com/RustAudio/cpal)
- [Web Audio API — AudioWorklet](https://developer.mozilla.org/en-US/docs/Web/API/AudioWorklet)
- [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [egui](https://github.com/emilk/egui)
- [Autocorrelation pitch detection](https://en.wikipedia.org/wiki/Autocorrelation#Efficient_computation)
