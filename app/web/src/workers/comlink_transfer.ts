import * as Comlink from "comlink";

Comlink.transferHandlers.set("BUFFER", {
  canHandle: (obj): obj is Uint8Array => obj instanceof Uint8Array,
  serialize: (obj: Uint8Array) => {
    console.log("WTF")
    return [
      obj,
      [obj.buffer],
    ];
  },
  deserialize: (obj: Uint8Array) => {
    try {
      console.log("WTF2")
      return obj.buffer
    } catch (err) {
      console.error("TRANSFER", err)
    }
  }
});