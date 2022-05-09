import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, shareReplay, from } from "rxjs";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { standardVisibilityTriggers$ } from "@/observable/visibility";

export interface GetEditFieldsArgs {
  objectKind: EditFieldObjectKind;
  id: number;
}

export interface GetEditFieldsRequest extends GetEditFieldsArgs, Visibility {
  workspaceId?: number;
}

export interface GetEditFieldsResponse {
  fields: EditFields;
}

const getEditFieldsCollection: {
  [objectKind: string]: {
    [key: string]: Observable<ApiResponse<GetEditFieldsResponse>>;
  };
} = {};

export function getEditFields(
  args: GetEditFieldsArgs,
): Observable<ApiResponse<GetEditFieldsResponse>> {
  if (
    getEditFieldsCollection[args.objectKind] &&
    getEditFieldsCollection[args.objectKind][args.id]
  ) {
    return getEditFieldsCollection[args.objectKind][args.id];
  }
  if (!getEditFieldsCollection[args.objectKind]) {
    getEditFieldsCollection[args.objectKind] = {};
  }
  getEditFieldsCollection[args.objectKind][args.id] = combineLatest([
    workspace$,
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([workspace, [visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      const request: GetEditFieldsRequest = { ...args, ...visibility };
      if (args.objectKind === EditFieldObjectKind.Component) {
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
        request.workspaceId = workspace.id;
      }
      return sdf.get<ApiResponse<GetEditFieldsResponse>>(
        "edit_field/get_edit_fields",
        request,
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getEditFieldsCollection[args.objectKind][args.id];
}
