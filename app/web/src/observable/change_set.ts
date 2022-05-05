import { ReplaySubject } from "rxjs";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { persistToSession } from "@/observable/session_state";
import {
  WsChangeSetApplied,
  WsChangeSetCanceled,
  WsChangeSetCreated,
  WsEvent,
} from "@/api/sdf/dal/ws_event";

/**
 * The currently active change set, or null if there isn't one.
 */
export const changeSet$ = new ReplaySubject<ChangeSet | null>(1);
changeSet$.next(null);
persistToSession("changeSet", changeSet$);

/**
 * The currently selected change set revision, or null if there isn't one.
 */
export const revision$ = new ReplaySubject<ChangeSet | null>(1);
revision$.next(null);
persistToSession("revision", revision$);

/**
 * Fired with the pk of the new change set when one is created.
 */
export const eventChangeSetCreated$ =
  new ReplaySubject<WsEvent<WsChangeSetCreated> | null>(1);
eventChangeSetCreated$.next(null);

/**
 * Fired with the pk of the new change set when one is applied.
 */
export const eventChangeSetApplied$ =
  new ReplaySubject<WsEvent<WsChangeSetApplied> | null>(1);
eventChangeSetApplied$.next(null);

/**
 * Fired with the pk of the new change set when one is canceled.
 */
export const eventChangeSetCanceled$ =
  new ReplaySubject<WsEvent<WsChangeSetCanceled> | null>(1);
eventChangeSetCanceled$.next(null);

/**
 * Fired when the user tries to edit something outside of an edit-session
 */
export const editButtonPulse$ = new ReplaySubject<boolean>(1);
editButtonPulse$.next(false);
