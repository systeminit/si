import {
  BehaviorSubject,
  combineLatest,
  debounceTime,
  from,
  Observable,
  shareReplay,
} from "rxjs";

import { tag } from "rxjs-spy/operators/tag";

import { switchMap } from "rxjs/operators";
import {
  changeSet$,
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
  eventChangeSetCreated$,
  eventChangeSetWritten$,
} from "@/observable/change_set";
import { NO_CHANGE_SET_PK, Visibility } from "@/api/sdf/dal/visibility";

export const showDeleted$ = new BehaviorSubject<boolean>(false);

export const visibility$: Observable<Visibility> = combineLatest([
  changeSet$,
  showDeleted$,
]).pipe(
  debounceTime(10),
  switchMap(([changeSet, showDeleted]) => {
    const visibility_change_set_pk = changeSet?.pk || NO_CHANGE_SET_PK;
    const visibility_deleted_at = showDeleted ? new Date() : undefined;
    const visibility: Visibility = {
      visibility_change_set_pk,
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
  eventChangeSetWritten$,
]).pipe(tag("standard-visibility"), shareReplay(1));
