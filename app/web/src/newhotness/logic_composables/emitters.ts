import mitt, { Emitter } from "mitt";

export interface KeyDetails {
  [key: string | symbol]: Pick<
    KeyboardEvent,
    | "key"
    | "ctrlKey"
    | "shiftKey"
    | "altKey"
    | "charCode"
    | "code"
    | "keyCode"
    | "metaKey"
    | "preventDefault"
  >;
}

export const keyEmitter: Emitter<KeyDetails> = mitt<KeyDetails>();

// Make sure we don't start the emitter more than once.
// This happens often when developing the system and causes redundant keydown evs
let keyEmitterStarted = false;

export const startKeyEmitter = (document: Document) => {
  if (keyEmitterStarted) return;
  keyEmitterStarted = true;
  document.addEventListener("keydown", (event: KeyboardEvent) => {
    const fromInput = ["INPUT", "TEXTAREA", "SELECT"].includes(
      (event.target as HTMLBodyElement)?.tagName,
    );

    if (!fromInput) {
      const isUpperCaseLetter = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".includes(
        event.key,
      );
      keyEmitter.emit(
        isUpperCaseLetter ? event.key.toLowerCase() : event.key,
        event,
      );
    }
  });
};

type AttributeDetails = {
  selectedPath: string;
  selectedDocs: { docs: string; link: string } | null;
};

export const attributeEmitter: Emitter<AttributeDetails> =
  mitt<AttributeDetails>();
