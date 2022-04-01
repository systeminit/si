import * as Rx from "rxjs";

import { SchematicKind } from "@/api/sdf/dal/schematic";
import {
  deploymentSelection$,
  componentSelection$,
  SelectedNode,
} from "../../state";
import { untilUnmounted } from "vuse-rx";
import { visibility$ } from "@/observable/visibility";

// This currently will only grow, if the user selects hundreds of panels
// the state for each panel will be preserved
export class SelectionManager {
  // Note: This should be a map with parentDeploymentNodeId,
  // but I noticed this too late into a series of refactors to fix bugs
  // I'm postponing this change
  selection: SelectedNode[];

  constructor() {
    this.selection = [];
    visibility$.pipe(untilUnmounted).subscribe((_) => {
      this.selection = [];
      deploymentSelection$.next([]);
      componentSelection$.next([]);
    });
  }

  // Selection should not be cleared when the schematic updates.
  select(
    selection: SelectedNode,
    selection$?: Rx.ReplaySubject<SelectedNode[]>,
  ): void {
    this.clearSelection(selection.parentDeploymentNodeId);

    for (const node of selection.nodes) {
      node.select();
      node.zIndex += 1;
    }

    const existing = this.selection.find(
      (n) => n.parentDeploymentNodeId === selection.parentDeploymentNodeId,
    );
    // Newer nodes always become the last element in the array
    if (existing) {
      existing.nodes = selection.nodes;
      const index = this.selection.indexOf(existing);
      this.selection.push(this.selection.splice(index, 1)[0]);

      if (selection$) {
        selection$.next([existing]);
      }
    } else {
      this.selection.push(selection);
      if (selection$) {
        selection$.next([selection]);
      }
    }
  }

  clearSelection(
    parentDeploymentNodeId: number | null,
    selection$?: Rx.ReplaySubject<SelectedNode[]>,
  ): void {
    for (const selection of this.selection) {
      if (selection.parentDeploymentNodeId === parentDeploymentNodeId) {
        for (const node of selection.nodes) {
          node.deselect();
          node.zIndex -= 1;
        }
        selection.nodes = [];
        break;
      }
    }

    if (selection$) {
      selection$.next(this.selection);
    }
  }

  selectionObserver(
    schematicKind: SchematicKind,
  ): Rx.ReplaySubject<SelectedNode[]> {
    switch (schematicKind) {
      case SchematicKind.Deployment:
        return deploymentSelection$;
      case SchematicKind.Component:
        return componentSelection$;
    }
    throw Error(`invalid schematic kind: ${schematicKind}`);
  }
}
