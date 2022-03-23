import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { from, mergeMap, Observable, take, tap } from "rxjs";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { switchToHead } from "@/service/change_set/switch_to_head";
import { changeSet$ } from "@/observable/change_set";

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
        switchToHead();
      }
    }),
    take(1),
  );
}
