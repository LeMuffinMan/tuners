
// https://github.com/RustAudio/cpal/issues/813
//
class MyProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.frameCount = 0;
  }
  
  process(inputs, outputs, parameters) {
    const input = inputs[0];
    
    if (input.length > 0) {
      const channelData = input[0];
      
      // Log toutes les 100 frames
      if (this.frameCount % 100 === 0) {
        console.log(`AudioWorklet: Processing ${channelData.length} samples`);
      }
      this.frameCount++;
      
      // Envoyer au Rust
      this.port.postMessage(channelData);
    } else {
      console.log("AudioWorklet: No input data");
    }
    
    return true;
  }
}

registerProcessor('my-processor', MyProcessor);
