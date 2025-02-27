export enum WorkerMessage {
  WASM_READY = "WASM_READY",
  CODEGEN = "CODEGEN",
  CODEGEN_COMPLETE = "CODEGEN_COMPLETE",
  LOAD_FILE = "LOAD_FILE",
  LOAD_FILE_COMPLETE = "LOAD_FILE_COMPLETE",
  CLEAR_FILE = "CLEAR_FILE",
}

export type WorkerMessageType =
  | { type: WorkerMessage.WASM_READY }
  | {
      type: WorkerMessage.CODEGEN;
      input: string;
      typename: string;
      options: Record<string, unknown>;
    }
  | {
      type: WorkerMessage.CODEGEN_COMPLETE;
      result: string;
      typename: string;
      options: Record<string, unknown>;
    }
  | { type: WorkerMessage.LOAD_FILE; file: File }
  | { type: WorkerMessage.LOAD_FILE_COMPLETE }
  | { type: WorkerMessage.CLEAR_FILE };
