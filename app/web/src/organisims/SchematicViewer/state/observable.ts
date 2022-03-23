import * as Rx from "rxjs";

import { Node } from "../Viewer/obj";

// These shouldn't be global, interaction manager should own them

export const deploymentSelection$ = new Rx.ReplaySubject<Array<Node> | null>(1);
deploymentSelection$.next(null);

export const componentSelection$ = new Rx.ReplaySubject<Array<Node> | null>(1);
componentSelection$.next(null);

// export const zoomMagnitude$ = new Rx.ReplaySubject<number | null>(1);
// zoomMagnitude$.next(null);

// export const zoomFactor$ = new Rx.ReplaySubject<number | null>(1);
// zoomFactor$.next(null);
