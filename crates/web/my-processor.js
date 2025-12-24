
class MyProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
  }

  process(inputs, outputs, parameters) {
    const inputChannels = inputs[0];
    if (inputChannels && inputChannels.length > 0) {
      const channelData = inputChannels[0]; // Float32Array

      const copy = new Float32Array(channelData.length);
      copy.set(channelData);

      this.port.postMessage(copy);
    }
    return true;
  }
}

registerProcessor("my-processor", MyProcessor);

