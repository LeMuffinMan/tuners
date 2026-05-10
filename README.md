# Tune.rs

<table>
<tr>
<td><h3>Real-time audio DSP in Rust — native + browser, same pipeline</h3></td>
<td align="right">
  <strong><a href="https://lemuffinman.github.io/tuners">▶ Try it live</a></strong>
</td>
</tr>
</table>

---

I built this to explore what real-time audio constraints actually look like in Rust. The premise: an instrument tuner. The real question: can the same DSP pipeline run identically on desktop and in a browser, with the constraints that come with it?

The hard constraint on the audio side: the capture callback runs on a dedicated thread (CPAL, native) or in an AudioWorklet rendering thread (WASM). Both contexts forbid allocation, blocking, and shared mutable state. On native you'd reach for a mutex — but that risks a priority inversion and a dropout. On WASM there is no SharedArrayBuffer available. In both cases, the solution is the same: a single-producer / single-consumer lock-free ring buffer. The audio side pushes raw `f32` samples; the DSP drains and computes each frame. No contention, no copies past the boundary.

The `AudioBackend` trait makes the rest of the system blind to which target is running. `cfg` gates are confined to backend selection and a few initialization paths. DSP and GUI see neither CPAL nor Web Audio — just a `Consumer<f32>`.

---

## What it does

- **Instrument tuner** — pitch detection in real time, note name and cents deviation from equal temperament
- **Waveform display** — live oscilloscope view of the captured signal
- **RMS meter** — energy envelope over time
- **CLI mode** — note + Hz output in the terminal (`--ui cli --visualizer freq`)
- **Native + WASM** — same DSP core, two targets, one codebase

---

## Architecture

```
crates/
├── audio/   — AudioBackend trait + NativeAudioBackend (CPAL) + WasmAudioBackend (Web Audio API)
├── dsp/     — DigitalSignalProcessor: RMS, waveform, autocorrelation, note + cents
├── gui/     — egui UI: desktop layout + mobile layout, render
├── native/  — native binary entry point, clap CLI parsing
└── wasm/    — WASM entry point (wasm-bindgen), AudioWorklet JS bridge
```

**Execution pipeline:**

```
Native:  CPAL callback → RingBuffer (rtrb) → DSP → GUI / CLI
WASM:    AudioWorklet (JS) → MessagePort → RingBuffer (rtrb) → DSP → GUI
```

---

## Key design decisions

### Lock-free boundary

Audio callbacks are real-time contexts: no allocation, no blocking, no mutex. The boundary between capture and processing is a single-producer / single-consumer lock-free ring buffer (`rtrb`, capacity 96 000 samples ≈ 2s at 48 kHz).

- **Producer** — owned by the audio callback, pushes raw `f32` samples
- **Consumer** — owned by the DSP, drained each frame

This works identically on both targets. On native, CPAL's callback pushes samples from its thread. On WASM, the AudioWorklet forwards batches via `MessagePort → Float32Array` — no `SharedArrayBuffer` needed.

### AudioBackend trait

Both targets implement the same interface:

```rust
trait AudioBackend {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    fn sample_rate(&self) -> f32;
}
```

- `NativeAudioBackend` — CPAL stream, `Producer<f32>` moved into the callback closure (ownership enforces the real-time contract: only the callback touches the producer)
- `WasmAudioBackend` — async init, AudioWorklet module loaded via promise, samples received through `MessagePort → Float32Array` → ring buffer

### Pitch detection

Autocorrelation on a 1024+ sample buffer, bounded to the musical range (80 Hz–1000 Hz). Peak lag → frequency → note name + cents deviation from equal temperament (A4 = 440 Hz). Not FFT-based — the intent was to understand the algorithm directly, not reach production-level accuracy.

| Feature | Method |
|---|---|
| RMS | `sqrt(Σs² / n)` |
| Waveform | Downsampled window of raw samples |
| Frequency | Autocorrelation, bounded lag, parabolic interpolation |
| Note | 12-TET from frequency, A4 = 440 Hz |
| Cents | Deviation from nearest semitone |

### GUI

The GUI crate (egui/eframe) adapts its layout at runtime:

- **Desktop** — control panel (start/stop, feature selector) + central visualization
- **Mobile** — larger fonts, stacked layout, touch-friendly controls

The same `TunerApp` struct handles both layouts. `cfg` gates are absent from GUI code — layout selection is runtime, not compile-time.

---

## Build & Run

### Native

```bash
# GUI (default)
cargo run -p tuners_native

# CLI — note + Hz in terminal
cargo run -p tuners_native -- --ui cli --visualizer freq

# CLI — RMS bar
cargo run -p tuners_native -- --ui cli --visualizer rms
```

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
- [Autocorrelation pitch detection](https://en.wikipedia.org/wiki/Autocorrelation)
- [egui](https://github.com/emilk/egui)
