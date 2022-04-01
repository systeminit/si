import * as Rx from "rxjs";

import { Node } from "../Viewer/obj";

// These shouldn't be global, interaction manager should own them

export interface SelectedNode {
  // Deployments never have a parentDeploymentNodeId, Components always have
  parentDeploymentNodeId: number | null;
  nodes: Array<Node>;
}

export const deploymentSelection$ = new Rx.ReplaySubject<SelectedNode[]>(1);
deploymentSelection$.next([]);

export const componentSelection$ = new Rx.ReplaySubject<SelectedNode[]>(1);
componentSelection$.next([]);

// export const zoomMagnitude$ = new Rx.ReplaySubject<number | null>(1);
// zoomMagnitude$.next(null);

// export const zoomFactor$ = new Rx.ReplaySubject<number | null>(1);
// zoomFactor$.next(null);
