import {
  BehaviorSubject,
  combineLatest,
  debounceTime,
  from,
  Observable,
  shareReplay,
} from "rxjs";
import {
  changeSet$,
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
  eventChangeSetCreated$,
} from "@/observable/change_set";
import {
  editSession$,
  editSessionWritten$,
  eventEditSessionSaved$,
} from "@/observable/edit_session";
import { switchMap } from "rxjs/operators";
import {
  NO_CHANGE_SET_PK,
  NO_EDIT_SESSION_PK,
  Visibility,
} from "@/api/sdf/dal/visibility";

export const showDeleted$ = new BehaviorSubject<boolean>(false);

export const visibility$: Observable<Visibility> = combineLatest([
  changeSet$,
  editSession$,
  showDeleted$,
]).pipe(
  debounceTime(10),
  switchMap(([changeSet, editSession, showDeleted]) => {
    const visibility_change_set_pk = changeSet?.pk || NO_CHANGE_SET_PK;
    const visibility_edit_session_pk = editSession?.pk || NO_EDIT_SESSION_PK;
    const visibility_deleted_at = showDeleted ? new Date() : undefined;
    const visibility: Visibility = {
      visibility_change_set_pk,
      visibility_edit_session_pk,
    };
    if (visibility_deleted_at) {
      visibility.visibility_deleted_at = visibility_deleted_at;
    }
    return from([visibility]);
  }),
  shareReplay(1),
);

export const standardVisibilityTriggers$ = combineLatest([
  visibility$,
  eventChangeSetCreated$,
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
  eventEditSessionSaved$,
  editSessionWritten$,
]).pipe(shareReplay(1));
