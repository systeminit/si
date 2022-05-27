import * as Rx from "rxjs";

import { nodeSelection$, SelectedNode } from "../../state";
import { untilUnmounted } from "vuse-rx";
import { schematicData$ } from "@/observable/schematic";

// This currently will grow indefinitely, if the user selects hundreds of panels
// the state for each panel will be preserved
export class SelectionManager {
  // Note: This should be a map with parentDeploymentNodeId,
  // but I noticed this too late into a series of refactors to fix bugs
  // I'm postponing this change
  selection: SelectedNode[];

  constructor() {
    this.selection = [];

    // We need to clean the selection if the node doesn't exist anymore
    // Happens when visibility changes
    schematicData$.pipe(untilUnmounted).subscribe((schematic) => {
      if (!schematic) return;

      const remove: Array<number | undefined> = [];
      for (const selection of this.selection) {
        const selectedNode = selection.nodes[0];
        if (!selectedNode || selectedNode.id === -1) continue;

        let shouldRemove = true;
        for (const node of schematic.nodes) {
          if (selectedNode.id === node.id) {
            shouldRemove = false;
            break;
          }
        }
        if (shouldRemove) {
          remove.push(selection.parentDeploymentNodeId);
        }
      }

      if (remove.length > 0) {
        this.selection = this.selection.filter(
          (selected) => !remove.includes(selected.parentDeploymentNodeId),
        );

        nodeSelection$.next(this.selection);
      }
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
    if (existing) {
      existing.nodes = selection.nodes;
    } else {
      this.selection.push(selection);
    }

    if (selection$) {
      selection$.next(this.selection);
    }
  }

  clearSelection(
    parentDeploymentNodeId?: number,
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
}
