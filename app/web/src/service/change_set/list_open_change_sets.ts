import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { LabelList } from "@/api/sdf/dal/label_list";
import { combineLatest, Observable, shareReplay } from "rxjs";
import { switchMap } from "rxjs/operators";
import {
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
  eventChangeSetCreated$,
} from "@/observable/change_set";

interface ListOpenChangesetsResponse {
  list: LabelList<number>;
}

/**
 * The list of open change sets, refreshed when the right events trigger, debounced, and
 * shared. Always populated once it's been called, if there are subscribers.
 */
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
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.get<ApiResponse<ListOpenChangesetsResponse>>(
        "change_set/list_open_change_sets",
      );
    },
  ),
  shareReplay({ bufferSize: 1, refCount: true }),
);

export function listOpenChangeSets(): Observable<
  ApiResponse<ListOpenChangesetsResponse>
> {
  return changeSetsOpenList$;
}
