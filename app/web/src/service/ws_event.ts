import { WsEvent } from "@/api/sdf/dal/ws_event";
import { ReplaySubject, BehaviorSubject } from "rxjs";
import {
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
  eventChangeSetCreated$,
} from "@/observable/change_set";
import { eventSchemaCreated$ } from "@/observable/schema";

const eventMap: {
  [E in WsEvent["payload"]["kind"]]: BehaviorSubject<any> | ReplaySubject<any>;
} = {
  ChangeSetCreated: eventChangeSetCreated$,
  ChangeSetApplied: eventChangeSetApplied$,
  ChangeSetCanceled: eventChangeSetCanceled$,
  SchemaCreated: eventSchemaCreated$,
};

export function dispatch(wsEvent: WsEvent) {
  const obs$ = eventMap[wsEvent.payload.kind];
  obs$.next(wsEvent.payload.data);
}

export function sendEvent(payload: WsEvent["payload"]) {
  const obs$ = eventMap[payload.kind];
  obs$.next(payload.data);
}

export const WsEventService = {
  dispatch,
  sendEvent,
};
