import * as Comlink from "comlink";

Comlink.transferHandlers.set("BUFFER", {
  canHandle: (obj): obj is ArrayBuffer => obj instanceof ArrayBuffer,
  serialize: (obj: ArrayBuffer) => {
    return [
      obj,
      [obj],
    ];
  },
  deserialize: (obj: ArrayBuffer) => obj,
});