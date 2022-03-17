import { WsEvent, WsPayloadKinds } from "@/api/sdf/dal/ws_event";
import { BehaviorSubject, ReplaySubject } from "rxjs";
import {
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
  eventChangeSetCreated$,
} from "@/observable/change_set";
import { eventEditSessionSaved$ } from "@/observable/edit_session";
import { eventResourceSynced$ } from "@/observable/resource";
import { eventSecretCreated$ } from "@/observable/secret";

const eventMap: {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [E in WsPayloadKinds["kind"]]: BehaviorSubject<any> | ReplaySubject<any>;
} = {
  ChangeSetCreated: eventChangeSetCreated$,
  ChangeSetApplied: eventChangeSetApplied$,
  ChangeSetCanceled: eventChangeSetCanceled$,
  EditSessionSaved: eventEditSessionSaved$,
  ResourceSynced: eventResourceSynced$,
  SecretCreated: eventSecretCreated$,
};

export function dispatch(wsEvent: WsEvent<WsPayloadKinds>) {
  const obs$ = eventMap[wsEvent.payload.kind];
  obs$.next(wsEvent);
}

export const WsEventService = {
  dispatch,
};
