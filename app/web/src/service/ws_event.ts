import { WsEvent, WsPayloadKinds } from "@/api/sdf/dal/ws_event";
import { BehaviorSubject, ReplaySubject } from "rxjs";
import {
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
  eventChangeSetCreated$,
} from "@/observable/change_set";
import { eventEditSessionSaved$ } from "@/observable/edit_session";

const eventMap: {
  [E in WsPayloadKinds["kind"]]: BehaviorSubject<any> | ReplaySubject<any>;
} = {
  ChangeSetCreated: eventChangeSetCreated$,
  ChangeSetApplied: eventChangeSetApplied$,
  ChangeSetCanceled: eventChangeSetCanceled$,
  EditSessionSaved: eventEditSessionSaved$,
};

export function dispatch(wsEvent: WsEvent<WsPayloadKinds>) {
  const obs$ = eventMap[wsEvent.payload.kind];
  obs$.next(wsEvent);
}

export const WsEventService = {
  dispatch,
};
