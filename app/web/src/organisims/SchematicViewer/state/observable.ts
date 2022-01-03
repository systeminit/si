import * as Rx from "rxjs";

import { Node } from "../Viewer/obj";

export const selection$ = new Rx.ReplaySubject<Array<Node> | null>(1);
selection$.next(null);

// export const zoomMagnitude$ = new Rx.ReplaySubject<number | null>(1);
// zoomMagnitude$.next(null);

// export const zoomFactor$ = new Rx.ReplaySubject<number | null>(1);
// zoomFactor$.next(null);
