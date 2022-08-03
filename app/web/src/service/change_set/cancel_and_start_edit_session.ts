import Bottle from "bottlejs";
import { Observable, tap, combineLatest, switchMap, take, from } from "rxjs";
import { ApiResponse, SDF } from "@/api/sdf";
import { EditSession } from "@/api/sdf/dal/edit_session";
import { editSession$ } from "@/observable/edit_session";
import { editMode$ } from "@/observable/edit_mode";
import { currentChangeSet } from "@/service/change_set/current_change_set";
import { currentEditSession } from "@/service/change_set/current_edit_session";

interface CancelAndStartEditSessionResponse {
  editSession: EditSession;
}

export function cancelAndStartEditSession(): Observable<
  ApiResponse<CancelAndStartEditSessionResponse>
> {
  return combineLatest([currentChangeSet(), currentEditSession()]).pipe(
    take(1),
    switchMap(([currentChangeSet, currentEditSession]) => {
      if (currentChangeSet && currentEditSession) {
        const bottle = Bottle.pop("default");
        const sdf: SDF = bottle.container.SDF;

        return sdf
          .post<ApiResponse<CancelAndStartEditSessionResponse>>(
            "change_set/cancel_and_start_edit_session",
            {
              changeSetPk: currentChangeSet.pk,
              editSessionPk: currentEditSession.pk,
            },
          )
          .pipe(
            tap((response) => {
              if (!response.error) {
                editSession$.next(response.editSession);
                editMode$.next(true);
              }
            }),
          );
      } else {
        const response = {
          error: {
            statusCode: 42,
            message:
              "either a current change set or a current edit session is not set",
            code: 400,
          },
        };
        return from([response]);
      }
    }),
    take(1),
  );
}
