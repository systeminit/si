import Bottle from "bottlejs";
import { Observable, tap, combineLatest, switchMap, take, from } from "rxjs";
import { ApiResponse, SDF } from "@/api/sdf";
import { EditSession } from "@/api/sdf/dal/edit_session";
import { currentChangeSet } from "@/service/change_set/current_change_set";
import { currentEditSession } from "@/service/change_set/current_edit_session";
import { switchToHead } from "@/service/change_set/switch_to_head";

interface SaveEditSessionAndApplyChangeSetResponse {
  editSession: EditSession;
}

export function saveEditSessionAndApplyChangeSet(): Observable<
  ApiResponse<SaveEditSessionAndApplyChangeSetResponse>
> {
  return combineLatest([currentChangeSet(), currentEditSession()]).pipe(
    take(1),
    switchMap(([currentChangeSet, currentEditSession]) => {
      if (currentChangeSet && currentEditSession) {
        const bottle = Bottle.pop("default");
        const sdf: SDF = bottle.container.SDF;

        console.log("HELLO");
        return sdf
          .post<ApiResponse<SaveEditSessionAndApplyChangeSetResponse>>(
            "change_set/save_edit_session_and_apply_change_set",
            {
              changeSetPk: currentChangeSet.pk,
              editSessionPk: currentEditSession.pk,
            },
          )
          .pipe(
            tap((response) => {
              if (!response.error) {
                switchToHead();
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
