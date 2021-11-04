import * as Rx from "rxjs";

import { Node } from "../Viewer/geo";

export const selection$ = new Rx.ReplaySubject<Array<Node> | null>(1);
selection$.next(null);

export const zoomMagnitude$ = new Rx.ReplaySubject<number | null>(1);
selection$.next(null);
