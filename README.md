# Tune.rs

This project is a hands-on exploration of real-time audio programming in Rust, with a strong focus on clean architecture, cross-platform design (native + WASM), and separation of concerns.

It is a small audio playground capable of:
 * capturing live audio input
 * computing basic DSP features (RMS, waveform, frequency / tuner)
 * visualizing the results through:
    * native GUI
    * CLI
    * web interface (WASM + AudioWorklet)

The same audio and DSP core is shared across all targets.

## 1. Building a real-time audio system

  * Capturing audio input in real time
  * Transferring audio samples safely between execution contexts
  * Computing DSP features from a continuous signal
  * Feeding processed data to different frontends (GUI / CLI / Web)

This required strict respect of real-time constraints, where architectural decisions directly impact performance and stability.

## 2. Designing a modular and scalable architecture

As i realized to late on my last rust project how architecture and design are important, i wanted to build a scalable and clean architecture from start this time.
The project is organized as autonomous crates with clear responsibilities:

├── crates
│   ├── audio
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── audio_bridge.rs
│   │       ├── backend
│   │       │   ├── mod.rs
│   │       │   ├── native.rs
│   │       │   └── wasm.rs
│   │       └── lib.rs
│   ├── cli
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── lib.rs
│   │       └── visualizer.rs
│   ├── dsp
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── lib.rs
│   ├── gui
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── lib.rs
│   │       ├── panels.rs
│   │       ├── render.rs
│   │       └── ui.rs
│   ├── native
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── main.rs
│   └── wasm
│       ├── Cargo.toml
│       ├── index.html
│       ├── my-processor.js
│       └── src
│           └── lib.rs

  * Audio: audio acquisition and backend abstraction (native / WASM)
  * Cli : enumarates features, and using clap to parse arguments 
  * DSP: signal processing on audio samples (RMS, waveform, frequency)
  * Gui : eframe and egui to provide a simple gui 

This structure allows:
  * multiple targets (native / WASM)
  * multiple frontends (GUI / CLI)
  * a shared audio and DSP core

Thus, we can summarize our exeuction model as such : 

Native:
Audio Callback (CPAL) -> RingBuffer -> DSP -> UI

WASM:
AudioWorklet (JS) -> MessagePort -> RingBuffer -> DSP -> UI

## 3. Abstracting audio backends with traits

To avoid code duplication and platform-specific logic leaking everywhere, the audio layer is abstracted behind a trait:
  * A single AudioBackend interface
  * Different implementations:
    * Native: CPAL audio callback, abstracting platform-specific audio APIs through a Rust interface.
    * Web: Web Audio API + AudioWorklet communicating with rust

Despite radically different APIs and execution models, both backends:
  * push raw samples into a shared buffer
  * expose the same lifecycle (start / stop)
  * keep DSP and UI platform-agnostic

I tried to use cfg only when i had no alternative.

## 4. Lock-free communication with a ring buffer

The core architectural challenge of this project was sharing audio data between real-time callbacks and the rest of the system.

The first compilation in native to explore CPAL basics worked fine even with mutex.
Since I learnt threads and mutex in C, that was quite straight forward and not too unfamiliar.
But soon, I realized it would not be what I need as this project grows : 
  * performance cost
  * latency potential impact
  * WASM threading constraints : 
    * AudioWorklet provide a dedicated audio thread, but not a WASM thread I can share directly with my main thread using SharedArryBuffer
    * The communication must be with MessagePort 

By browsing internet looking for alternatives in different articles or projects : 
  * https://medium.com/@nathanbcrocker/implementing-a-lock-free-ring-buffer-in-go-ee36bba220ea
  * https://dev.to/drsh4dow/the-joy-of-the-unknown-exploring-audio-streams-with-rust-and-circular-buffers-494d

Led me to choose the lock free ring buffer solution, using real-time ring buffer since it's built for real time conditions : 
rtrb provides a single-producer / single-consumer lock-free ring buffer, which guarantees bounded execution time and avoids blocking in real-time audio callbacks.
The ring buffer acts as the architectural boundary between real-time audio capture and non-real-time computation.
  * Producer
    * owned by the audio callback (CPAL or AudioWorklet)
    * pushes samples with minimal overhead
  * Consumer
    * owned by the DSP layer
    * pulls samples when computing features

This design:
  * avoids blocking in the audio callback
  * works both in native and WASM environments (my first concern in the beginning)
  * cleanly separates real-time code from computation and rendering

## 5. Ownership, closures, and callbacks in Rust

Working with audio callbacks and WASM required to deepen my understanding of:
  * Ownership transfer into closures (move)
  * Long-lived callbacks (CPAL streams, JS event handlers)
  * Safe interaction between Rust and JavaScript memory models
  * Trade-offs around Send, Sync, Arc, and Mutex

Concrete examples include:

  * moving the Producer<f32> into audio callbacks
  * preventing Rust closures from being dropped when called from JS
  * keeping callbacks allocation-free and deterministic

## 6. Audio and DSP fundamentals

Real-time audio constraints
  * No allocations inside audio callbacks
  * Stable timing and low latency 
  * Minimal work in the capture thread

Signal processing basics, extracting usable data from raw audio:
  * RMS
  * waveform
  * frequency detection (tuner)

Understanding trade-offs:
  * latency vs accuracy
  * CPU usage vs precision
  * Optimization mindset
  * deciding where and when to compute
  * choosing between raw samples vs derived features
  * balancing simplicity, performance, and correctness

## 7. WASM and Web Audio

I already explored WASM with rust coding a multi-target chessgame. So i was not starting from scratch.
Using Web Audio API and AudioWorklet for audio input was new :
  * Async loading and initialization of audio worklets
  * Interoperability between Rust (WASM) and JavaScript

Exploring WASM limitations in this context:
  * threading : the AudioWorklet is not a thread i can access from rust
  * timing guarantees : since web API makes one more step in between the audio interface and my code
  * browser-controlled audio configuration (sample rate, buffer size)

Despite these constraints, the project maintains:
  * a unified DSP pipeline
  * shared visualization logic
  * a single codebase for native and web targets

Key takeaways
  * Real-time audio programming forces architectural discipline
  * Lock-free communication is often necessary, not optional
  * Traits enable clean separation between platform-specific code and core logic
  * Rust is particularly well-suited for systems programming where correctness and performance matter
  * Designing for multiple targets early helps avoid technical debt

Possible future improvements
  * Dedicated DSP thread on native targets : i want to reach wasm real time audio limits and compare performances with rust
  * Adaptive buffer sizes and dynamic sample rate handling : WebAPI can be less permissive as CPAL.
  * More advanced DSP (FFT-based analysis)
  * Proper lifecycle management for JS callbacks (drop / cleanup) : This intentionally leaks the closure for the lifetime of the application. Proper lifecycle management would be required in a production setting.  * Performance benchmarks and profiling : using cargo for tests and bench, I want to dedicate a time to measure each opitmisation gain as the DSP grows

