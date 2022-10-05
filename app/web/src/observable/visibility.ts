import {
  combineLatest,
  debounceTime,
  from,
  Observable,
  shareReplay,
} from "rxjs";

import { tag } from "rxjs-spy/operators/tag";

import { switchMap } from "rxjs/operators";
import { NO_CHANGE_SET_PK, Visibility } from "@/api/sdf/dal/visibility";
import { changeSet$ } from "@/service/change_set";

export const visibility$: Observable<Visibility> = combineLatest([
  changeSet$,
]).pipe(
  debounceTime(10),
  switchMap(([changeSet]) => {
    const visibility_change_set_pk = changeSet?.pk || NO_CHANGE_SET_PK;
    const visibility: Visibility = {
      visibility_change_set_pk,
    };
    return from([visibility]);
  }),
  shareReplay(1),
);

export const standardVisibilityTriggers$ = combineLatest([
  visibility$,
  changeSet$,
]).pipe(tag("standard-visibility"), shareReplay(1));
