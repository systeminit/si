import { selection$ } from "../../state";
import { Node } from "../obj";

export class SelectionManager {
  selection: Array<Node>;

  constructor() {
    this.selection = [];
  }

  // Selection should not be cleared when the schematic updates.
  select(node: Node): void {
    if (this.selection.length > 0) {
      this.clearSelection();
    }
    node.select();
    node.zIndex += 1;
    this.selection.push(node);
    selection$.next(this.selection);
  }

  clearSelection(): void {
    for (let i = 0; i < this.selection.length; i++) {
      this.selection[i].deselect();
      this.selection[i].zIndex -= 1;
      delete this.selection[i];
    }
    this.selection = [];
    selection$.next(null);
  }
}
