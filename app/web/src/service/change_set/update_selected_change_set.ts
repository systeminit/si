import Bottle from "bottlejs";
import { Observable, tap, mergeMap, take } from "rxjs";
import { ApiResponse, SDF } from "@/api/sdf";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { EditSession } from "@/api/sdf/dal/edit_session";
import { currentEditSession } from "@/service/change_set/current_edit_session";
import { changeSet$ } from "@/observable/change_set";
import { editSession$ } from "@/observable/edit_session";
import { editMode$ } from "@/observable/edit_mode";

export interface UpdatedSelectedChangeSetArgs {
  nextChangeSetPk: number;
}

export interface UpdatedSelectedChangeSetRequest
  extends UpdatedSelectedChangeSetArgs {
  currentEditSessionPk?: number;
}

interface UpdateSelectedChangeSetResponse {
  changeSet: ChangeSet;
  editSession: EditSession;
}

export function updateSelectedChangeSet(
  args: UpdatedSelectedChangeSetArgs,
): Observable<ApiResponse<UpdateSelectedChangeSetResponse>> {
  return currentEditSession().pipe(
    take(1),
    mergeMap((currentEditSession) => {
      const request: UpdatedSelectedChangeSetRequest = args;
      if (currentEditSession) {
        request.currentEditSessionPk = currentEditSession.pk;
      }

      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf
        .post<ApiResponse<UpdateSelectedChangeSetResponse>>(
          "change_set/update_selected_change_set",
          request,
        )
        .pipe(
          tap((response) => {
            if (!response.error) {
              changeSet$.next(response.changeSet);
              editSession$.next(response.editSession);
              editMode$.next(true);
            }
          }),
        );
    }),
    take(1),
  );
}
