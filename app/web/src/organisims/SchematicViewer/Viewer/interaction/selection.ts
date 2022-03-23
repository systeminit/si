import * as Rx from "rxjs";

import { Node } from "../obj";

export class SelectionManager {
  selection: Array<Node>;

  constructor() {
    this.selection = [];
  }

  // Selection should not be cleared when the schematic updates.
  select(node: Node, selection$?: Rx.ReplaySubject<Array<Node> | null>): void {
    if (this.selection.length > 0) {
      this.clearSelection();
    }

    node.select();
    node.zIndex += 1;
    this.selection.push(node);
    if (selection$) selection$.next(this.selection);
  }

  clearSelection(selection$?: Rx.ReplaySubject<Array<Node> | null>): void {
    for (const selection of this.selection) {
      selection.deselect();
      selection.zIndex -= 1;
    }
    this.selection = [];

    if (selection$) selection$.next(null);
  }
}
