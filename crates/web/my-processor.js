
// https://github.com/RustAudio/cpal/issues/813
//
function computeRMS(buffer) {
    let sum = 0;
    for (let i = 0; i < buffer.length; i++) {
        sum += buffer[i] * buffer[i];
    }
    return Math.sqrt(sum / buffer.length);
}

class MyProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
    }

    process(inputs, outputs, parameters) {
        const inputChannels = inputs[0];
        if (inputChannels && inputChannels.length > 0) {
            const channelData = inputChannels[0];
            const rms = computeRMS(channelData);

            this.port.postMessage({ rms });

            // this.port.postMessage({ debug: `RMS computed: ${rms}` });
        }
        return true;
    }
}

registerProcessor("my-processor", MyProcessor);

