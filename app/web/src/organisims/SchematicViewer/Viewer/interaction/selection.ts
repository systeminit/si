import * as Rx from "rxjs";

import { SchematicKind } from "@/api/sdf/dal/schematic";
import * as OBJ from "../obj";
import { deploymentSelection$, componentSelection$ } from "../../state";

export class SelectionManager {
  selection: Array<OBJ.Node>;

  constructor() {
    this.selection = [];
  }

  // Selection should not be cleared when the schematic updates.
  select(
    node: OBJ.Node,
    selection$?: Rx.ReplaySubject<Array<OBJ.Node> | null>,
  ): void {
    if (this.selection.length > 0) {
      this.clearSelection();
    }

    node.select();
    node.zIndex += 1;
    this.selection.push(node);
    if (selection$) selection$.next(this.selection);
  }

  clearSelection(selection$?: Rx.ReplaySubject<Array<OBJ.Node> | null>): void {
    for (const selection of this.selection) {
      selection.deselect();
      selection.zIndex -= 1;
    }
    this.selection = [];

    if (selection$) selection$.next(null);
  }

  selectionObserver(
    schematicKind: SchematicKind,
  ): Rx.ReplaySubject<Array<OBJ.Node> | null> {
    switch (schematicKind) {
      case SchematicKind.Deployment:
        return deploymentSelection$;
      case SchematicKind.Component:
        return componentSelection$;
    }
    throw Error(`invalid schematic kind: ${schematicKind}`);
  }
}
