import {WorkerMessage} from "./WorkerMessage";

let typegenWasm;
let largeFileInput;

import("../../json_typegen_wasm/pkg").then(module => {
  typegenWasm = module;
  postMessage({
    type: WorkerMessage.WASM_READY
  });
});

onmessage = messageEvent => {
  const message = messageEvent.data;

  if (message.type === WorkerMessage.CODEGEN) {
    const input = largeFileInput || message.input;
    const result = typegenWasm.run(message.typename, input, JSON.stringify(message.options));
    postMessage({
      type: WorkerMessage.CODEGEN_COMPLETE,
      result,
      typename: message.typename,
      options: message.options,
    });
  } else if (message.type === WorkerMessage.LOAD_FILE) {
    const reader = new FileReader();
    reader.onload = (fileEvent) => {
      largeFileInput = fileEvent.target.result;
      postMessage({
        type: WorkerMessage.LOAD_FILE_COMPLETE,
      });
    }
    reader.readAsText(message.file);
  } else if (message.type === WorkerMessage.CLEAR_FILE) {
    largeFileInput = undefined;
  } else {
    console.warn("Unknown message to worker", messageEvent);
  }
};
