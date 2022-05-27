import * as Rx from "rxjs";

import { Node } from "../Viewer/obj";

// These shouldn't be global, interaction manager should own them

export interface SelectedNode {
  // Deployments never have a parentDeploymentNodeId, Components always have
  parentDeploymentNodeId?: number;
  nodes: Array<Node>;
}

export const nodeSelection$ = new Rx.ReplaySubject<SelectedNode[]>(1);
nodeSelection$.next([]);

export const lastSelectedNode$ = new Rx.ReplaySubject<Node | null>(1);
lastSelectedNode$.next(null);

export const lastSelectedDeploymentNode$ = new Rx.ReplaySubject<Node | null>(1);
lastSelectedNode$.next(null);

// export const zoomMagnitude$ = new Rx.ReplaySubject<number | null>(1);
// zoomMagnitude$.next(null);

// export const zoomFactor$ = new Rx.ReplaySubject<number | null>(1);
// zoomFactor$.next(null);
