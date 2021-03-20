import { PanelEvent, EventKind, EventPayload, SenderCtx } from "./PanelEvent";

const EVENT_ID = EventKind.RemoveTransientEdge;

interface RemoveTransientEdgeEventPayload extends EventPayload {}

export class RemoveTransientEdgeEvent extends PanelEvent {
  constructor(senderCtx: SenderCtx, payload: RemoveTransientEdgeEventPayload) {
    super(EVENT_ID, senderCtx);
  }

  static eventName(): string {
    return EVENT_ID;
  }
}
