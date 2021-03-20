import { PanelEventBus } from "../PanelEventBus";

export enum EventKind {
  RemoveTransientEdge = "removeTransientEdge",
}

export interface SenderCtx {
  senderId: string;
}

interface EventHandlerFunction {
  (): void;
}

export interface IEvent {
  kind: EventKind;
  senderCtx: SenderCtx;
  payload?: any;
}

export interface EventPayload {
  senderCtx: SenderCtx;
}

export abstract class PanelEvent {
  eventId: string;
  senderCtx: SenderCtx;

  constructor(eventId: string, senderCtx: SenderCtx) {
    this.eventId = eventId;
    this.senderCtx = senderCtx;
  }

  subscribe(fn: EventHandlerFunction) {
    PanelEventBus.$on(this.eventId, fn);
  }

  publish(payload?: EventPayload) {
    PanelEventBus.$emit(this.eventId, payload);
  }

  static eventName(): string {
    throw new Error(
      "You must provide a static eventName in your implementation of a panel event! Bug!",
    );
  }
}
