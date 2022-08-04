import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { EditSession } from "@/api/sdf/dal/edit_session";
import { changeSet$ } from "@/observable/change_set";
import { editSession$ } from "@/observable/edit_session";
import { editMode$ } from "@/observable/edit_mode";
import { currentEditSession } from "@/service/change_set/current_edit_session";
import { Observable, tap, take, mergeMap } from "rxjs";

interface CreateChangeSetArgs {
  changeSetName: string;
}

export interface CreateChangeSetRequest extends CreateChangeSetArgs {
  currentEditSessionPk?: number;
}

interface CreateChangeSetResponse {
  changeSet: ChangeSet;
  editSession: EditSession;
}

export function createChangeSet(
  args: CreateChangeSetArgs,
): Observable<ApiResponse<CreateChangeSetResponse>> {
  return currentEditSession().pipe(
    take(1),
    mergeMap((currentEditSession) => {
      const request: CreateChangeSetRequest = args;
      if (currentEditSession) {
        request.currentEditSessionPk = currentEditSession.pk;
      }

      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf
        .post<ApiResponse<CreateChangeSetResponse>>(
          "change_set/create_change_set",
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
