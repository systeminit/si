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
  >;
}

export const keyEmitter: Emitter<KeyDetails> = mitt<KeyDetails>();

export const startKeyEmitter = (document: Document) => {
  document.addEventListener("keydown", (event: KeyboardEvent) => {
    const fromInput = ["INPUT", "TEXTAREA", "SELECT"].includes(
      (event.target as HTMLBodyElement)?.tagName,
    );

    if (!fromInput) {
      keyEmitter.emit(event.key, event);
    }
  });
};

type AttributeDetails = {
  selectedPath: string;
  scrolled: void;
};

export const attributeEmitter: Emitter<AttributeDetails> =
  mitt<AttributeDetails>();
