import { WsEvent, WsPayloadKinds } from "@/api/sdf/dal/ws_event";
import { BehaviorSubject, ReplaySubject, Subject } from "rxjs";
import {
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
  eventChangeSetCreated$,
} from "@/observable/change_set";
import { eventEditSessionSaved$ } from "@/observable/edit_session";
import { eventResourceSynced$ } from "@/observable/resource";
import { eventCheckedQualifications$ } from "@/observable/qualification";
import { eventDependentValuesUpdated$ } from "@/observable/attribute_value";
import { eventCodeGenerated$ } from "@/observable/code";
import { eventSecretCreated$ } from "@/observable/secret";

const eventMap: {
  [E in WsPayloadKinds["kind"]]:  // eslint-disable-next-line @typescript-eslint/no-explicit-any
    | BehaviorSubject<any>
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    | ReplaySubject<any>
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    | Subject<any>;
} = {
  ChangeSetCreated: eventChangeSetCreated$,
  ChangeSetApplied: eventChangeSetApplied$,
  ChangeSetCanceled: eventChangeSetCanceled$,
  EditSessionSaved: eventEditSessionSaved$,
  ResourceSynced: eventResourceSynced$,
  CodeGenerated: eventCodeGenerated$,
  CheckedQualifications: eventCheckedQualifications$,
  UpdatedDependentValue: eventDependentValuesUpdated$,
  SecretCreated: eventSecretCreated$,
};

export function dispatch(wsEvent: WsEvent<WsPayloadKinds>) {
  const obs$ = eventMap[wsEvent.payload.kind];
  obs$.next(wsEvent);
}

export const WsEventService = {
  dispatch,
};
