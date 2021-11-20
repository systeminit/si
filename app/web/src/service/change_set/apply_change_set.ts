import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { editMode$ } from "@/observable/edit_mode";
import { from, mergeMap, Observable, take, tap } from "rxjs";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { changeSet$ } from "@/observable/change_set";
import { editSession$ } from "@/observable/edit_session";

/**
 * Returns the change set that has been applied, or null if no
 * change set existed to save in the first place.
 */
interface ApplyChangeSetResponse {
  changeSet: ChangeSet | null;
}

export function applyChangeSet(): Observable<
  ApiResponse<ApplyChangeSetResponse>
> {
  return changeSet$.pipe(
    take(1),
    mergeMap((changeSet) => {
      if (changeSet) {
        const bottle = Bottle.pop("default");
        const sdf: SDF = bottle.container.SDF;
        return sdf.post<ApiResponse<ApplyChangeSetResponse>>(
          "change_set/apply_change_set",
          {
            changeSetPk: changeSet.pk,
          },
        );
      } else {
        const response: ApiResponse<ApplyChangeSetResponse> = {
          changeSet: null,
        };
        return from([response]);
      }
    }),
    tap((response) => {
      if (!response.error) {
        changeSet$.next(null);
        editSession$.next(null);
        editMode$.next(false);
      }
    }),
    take(1),
  );
}
