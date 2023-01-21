import { Direction } from "../diagram_types";

/** convert arrow key codes into a simple direction (ex: "ArrowUp" -> "up") */
export function convertArrowKeyToDirection(keyCode: string) {
  const direction = {
    ArrowUp: "up",
    ArrowDown: "down",
    ArrowLeft: "left",
    ArrowRight: "right",
  }[keyCode];
  if (!direction) throw new Error("keyCode was not an arrow key");
  return direction as Direction;
}
