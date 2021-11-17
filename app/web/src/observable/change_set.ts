import {
  combineLatest,
  debounceTime,
  from,
  ReplaySubject,
  shareReplay,
} from "rxjs";
import { switchMap } from "rxjs/operators";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { ChangeSetService } from "@/service/change_set";
import { persistToSession } from "@/observable/session_state";

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
 * Fired with the id of the new change set when one is created.
 */
export const eventChangeSetCreated$ = new ReplaySubject<number | null>(1);
eventChangeSetCreated$.next(null);

/**
 * Fired with the id of the new change set when one is applied.
 */
export const eventChangeSetApplied$ = new ReplaySubject<number | null>(1);
eventChangeSetApplied$.next(null);

/**
 * Fired with the id of the new change set when one is canceled.
 */
export const eventChangeSetCanceled$ = new ReplaySubject<number | null>(1);
eventChangeSetCanceled$.next(null);

export const changeSetsOpenList$ = combineLatest([
  eventChangeSetCreated$,
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
]).pipe(
  switchMap(
    ([
      _eventChangeSetCreated,
      _eventChangeSetApplied,
      _eventChangeSetCanceled,
    ]) => {
      return from(ChangeSetService.listOpenChangeSets());
    },
  ),
  debounceTime(100),
  shareReplay(1),
);
