import { Node } from "../geo";
import { Position } from "../cg";

export function translateNode(node: Node, position: Position): void {
  node.x = position.x;
  node.y = position.y;
  node.updateTransform();
}
