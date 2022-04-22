import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable, take, tap } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { EditFieldObjectKind } from "@/api/sdf/dal/edit_field";
import { editSessionWritten$ } from "@/observable/edit_session";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { AttributeContext } from "@/api/sdf/dal/attribute";

export interface UpdateFromEditFieldArgs {
  objectKind: EditFieldObjectKind;
  objectId: number;
  editFieldId: string;
  value: unknown;
  attributeContext: AttributeContext;
  baggage?: unknown;
}

export interface UpdateFromEditFieldRequest
  extends UpdateFromEditFieldArgs,
    Visibility {
  workspaceId?: number;
}

export interface UpdateFromEditFieldResponse {
  success: boolean;
}

export function updateFromEditField(
  args: UpdateFromEditFieldArgs,
): Observable<ApiResponse<UpdateFromEditFieldResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return combineLatest([visibility$, workspace$]).pipe(
    take(1),
    switchMap(([visibility, workspace]) => {
      let request: UpdateFromEditFieldRequest;
      if (
        args.objectKind === EditFieldObjectKind.Component ||
        args.objectKind === EditFieldObjectKind.ComponentProp
      ) {
        if (_.isNull(workspace)) {
          return from([
            {
              error: {
                statusCode: 10,
                message: "cannot make call without a workspace; bug!",
                code: 10,
              },
            },
          ]);
        }
        request = {
          ...args,
          ...visibility,
          workspaceId: workspace.id,
        };
      } else {
        request = {
          ...args,
          ...visibility,
        };
      }
      return sdf
        .post<ApiResponse<UpdateFromEditFieldResponse>>(
          "edit_field/update_from_edit_field",
          request,
        )
        .pipe(
          tap((_response) => {
            editSessionWritten$.next(true);
          }),
        );
    }),
  );
}
