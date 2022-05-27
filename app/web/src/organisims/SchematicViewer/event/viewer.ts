import * as Rx from "rxjs";
import { SchematicNode } from "@/api/sdf/dal/schematic";

// Viewer event - sendind events to the viewer...

export enum ViewerEventKind {
  NODE_ADD = "node_add",
}

export interface NodeAddEventData {
  node: SchematicNode;
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

export class ViewerEventObservable {
  viewerEvent$: Rx.ReplaySubject<ViewerEvent | null>;
  constructor() {
    this.viewerEvent$ = new Rx.ReplaySubject<ViewerEvent | null>(1);
    this.viewerEvent$.next(null);
  }
}
