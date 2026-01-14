import mitt, { Emitter } from "mitt";
import { ref } from "vue";

export interface KeyDetails {
  [key: string | symbol]: Pick<
    KeyboardEvent,
    "key" | "ctrlKey" | "shiftKey" | "altKey" | "charCode" | "code" | "keyCode" | "metaKey" | "preventDefault"
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
    const fromInput = ["INPUT", "TEXTAREA", "SELECT"].includes((event.target as HTMLBodyElement)?.tagName);

    if (!fromInput) {
      // letter keys should be case insensitive
      const isUpperCaseLetter = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".includes(event.key);
      keyEmitter.emit(isUpperCaseLetter ? event.key.toLowerCase() : event.key, event);
    }
  });
};

type AttributeDetails = {
  selectedPath: { path: string; name: string };
  selectedDocs: { docs: string; link: string } | null;
};

export const attributeEmitter: Emitter<AttributeDetails> = mitt<AttributeDetails>();

export const windowWidthReactive = ref<number>(window.innerWidth);
export const windowHeightReactive = ref<number>(window.innerHeight);

export interface ResizeDetails {
  [resize: string | symbol]: Pick<Event, "preventDefault" | "target">;
}

export const windowResizeEmitter: Emitter<ResizeDetails> = mitt<ResizeDetails>();

let windowResizeEmitterStarted = false;

const onResize = (event: Event) => {
  windowWidthReactive.value = window.innerWidth;
  windowHeightReactive.value = window.innerHeight;

  windowResizeEmitter.emit("resize", event);
};

export const startWindowResizeEmitter = (window: Window) => {
  if (windowResizeEmitterStarted) return;
  windowResizeEmitterStarted = true;
  window.addEventListener("resize", onResize);
};

export interface MouseDetails {
  [key: string | symbol]: Pick<
    MouseEvent,
    "button" | "clientX" | "clientY" | "altKey" | "ctrlKey" | "metaKey" | "shiftKey" | "preventDefault" | "target"
  >;
}

const onClick = (event: MouseEvent) => {
  mouseEmitter.emit("click", event);
};
const onMouseDown = (event: MouseEvent) => {
  mouseEmitter.emit("mousedown", event);
};

export const mouseEmitter: Emitter<MouseDetails> = mitt<MouseDetails>();

let mouseEmitterStarted = false;

export const startMouseEmitters = (window: Window) => {
  if (mouseEmitterStarted) return;
  mouseEmitterStarted = true;
  window.addEventListener("click", onClick);
  window.addEventListener("mousedown", onMouseDown);
};
