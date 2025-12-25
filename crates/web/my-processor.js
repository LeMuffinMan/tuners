

function computeRMS(buffer) {
    let sum = 0;
    for (let i = 0; i < buffer.length; i++) {
        sum += buffer[i] * buffer[i];
    }
    return Math.sqrt(sum / buffer.length);
}

///my-processor.js define an AudioWorkletProcessor
///
///we define a custom AudioWorkletProcessor 
///Using it with the Web Audio API allows us to work on real time audio
///with a separate thread. Thus we can use the main thread for the DPS and the UI
class MyProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
  }

  //audio engine will keep calling process while it returns true
  process(inputs, outputs, parameters) {
    const inputChannels = inputs[0];
    if (inputChannels && inputChannels.length > 0) {
      //we want to read the first cannal data
      const channelData = inputChannels[0];
      const rms = computeRMS(channelData);

      //we don't want to send to main thread the channelData, since audio engine work with it
      //sending a copy prevent the main thread, our dsp and ui, to interfere with the audio engine 
      this.port.postMessage(rms);
    }
    return true; //this triggers again process
  }
}

registerProcessor("my-processor", MyProcessor);

