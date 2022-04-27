import { ReplaySubject } from "rxjs";
import { EditSession } from "@/api/sdf/dal/edit_session";
import { persistToSession } from "@/observable/session_state";
import { WsEditSessionSaved, WsEvent } from "@/api/sdf/dal/ws_event";

export const editSession$ = new ReplaySubject<EditSession | null>(1);
editSession$.next(null);
persistToSession("editSession", editSession$);

/**
 * Fired with the id of the new edit session when one is created.
 */
export const eventEditSessionSaved$ =
  new ReplaySubject<WsEvent<WsEditSessionSaved> | null>(1);
eventEditSessionSaved$.next(null);

/**
 * Fired with the id of the cancelled edit session.
 */
export const editSessionCanceled$ = new ReplaySubject<number | null>(1);
editSessionCanceled$.next(null);

/**
 * Fired when an edit session has been written to
 */
export const editSessionWritten$ = new ReplaySubject<true>(1);
editSessionWritten$.next(true);
