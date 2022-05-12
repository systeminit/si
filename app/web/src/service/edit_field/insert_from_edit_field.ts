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

export interface InsertFromEditFieldArgs {
  objectKind: EditFieldObjectKind;
  objectId: number;
  editFieldId: string;
  attributeContext: AttributeContext;
  key?: string;
  baggage?: unknown;
}

export interface InsertFromEditFieldRequest
  extends InsertFromEditFieldArgs,
    Visibility {
  workspaceId?: number;
}

export interface InsertFromEditFieldResponse {
  success: boolean;
}

export function insertFromEditField(
  args: InsertFromEditFieldArgs,
): Observable<ApiResponse<InsertFromEditFieldResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return combineLatest([visibility$, workspace$]).pipe(
    take(1),
    switchMap(([visibility, workspace]) => {
      let request: InsertFromEditFieldRequest;
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
        .post<ApiResponse<InsertFromEditFieldResponse>>(
          "edit_field/insert_from_edit_field",
          request,
        )
        .pipe(
          tap((response) => {
            editSessionWritten$.next(true);
            console.log({ response });
          }),
        );
    }),
  );
}
