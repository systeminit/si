import * as Rx from "rxjs";

import { SceneManager } from "@/organisims/SchematicViewer/Viewer/scene_manager";
import { Node } from "@/organisims/SchematicViewer/Viewer/obj/node";

// These shouldn't be global, interaction manager should own them

export interface SelectedNode {
  // Deployments never have a parentDeploymentNodeId, Components always have
  parentDeploymentNodeId: number | null;
  nodeIds: number[];
}

// For now we clear the old selection when adding a new one, but we could support multiselection without problems
export async function selectNode(
  nodeId: number,
  parentDeploymentNodeId: number | null,
) {
  const selections = await Rx.firstValueFrom(nodeSelection$);
  for (const selection of selections) {
    if (selection.parentDeploymentNodeId === parentDeploymentNodeId) {
      selection.nodeIds = [nodeId];
      nodeSelection$.next(selections);
      return;
    }
  }

  selections.push({
    parentDeploymentNodeId,
    nodeIds: [nodeId],
  });
  nodeSelection$.next(selections);
}

export async function clearSelection(parentDeploymentNodeId: number | null) {
  const selections = (await Rx.firstValueFrom(nodeSelection$)).filter(
    (selection) => selection.parentDeploymentNodeId !== parentDeploymentNodeId,
  );
  nodeSelection$.next(selections);
}

export async function findSelectedNodes(
  sceneManager: SceneManager,
  parentDeploymentNodeId: number | null,
): Promise<Node[]> {
  const selections = await Rx.firstValueFrom(nodeSelection$);
  const sceneNodes = sceneManager.group.nodes.children as Node[];
  const nodes: Node[] = [];

  const selected = selections.find(
    (s) => s.parentDeploymentNodeId === parentDeploymentNodeId,
  );
  for (const nodeId of selected?.nodeIds ?? []) {
    if (nodeId === -1) continue;

    const node = sceneNodes.find((n) => n.id === nodeId);

    if (node) {
      nodes.push(node);
    } else {
      throw new Error("unable to find selected node: " + nodeId);
    }
  }
  return nodes;
}

export const nodeSelection$ = new Rx.ReplaySubject<SelectedNode[]>(1);
nodeSelection$.next([]);

export const lastSelectedNode$ = new Rx.ReplaySubject<Node | null>(1);
lastSelectedNode$.next(null);

export const lastSelectedDeploymentNode$ = new Rx.ReplaySubject<Node | null>(1);
lastSelectedNode$.next(null);
