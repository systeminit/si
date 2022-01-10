import * as Rx from "rxjs";
import * as MODEL from "../model";

// Viewer event - sendind events to the viewer...

export enum ViewerEventKind {
  NODE_ADD = "node_add",
}

export interface NodeAddEventData {
  node: MODEL.Node;
  schemaId: number;
}

export class NodeAddEvent {
  kind: ViewerEventKind;
  data: NodeAddEventData;

  constructor(data: NodeAddEventData) {
    this.kind = ViewerEventKind.NODE_ADD;
    this.data = data;
  }
}

export type ViewerEvent = NodeAddEvent;

// export const viewerEvent$ = new Rx.ReplaySubject<ViewerEvent | null>(1);
// viewerEvent$.next(null);

export class ViewerEventObservable {
  viewerEvent$: Rx.ReplaySubject<ViewerEvent | null>;
  constructor() {
    this.viewerEvent$ = new Rx.ReplaySubject<ViewerEvent | null>(1);
    this.viewerEvent$.next(null);
  }
}
