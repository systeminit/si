import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, shareReplay, from } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";

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
    standardVisibilityTriggers$,
    workspace$,
  ]).pipe(
    switchMap(([[visibility], workspace]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      let request: GetEditFieldsRequest;
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
      const response = sdf.get<ApiResponse<GetEditFieldsResponse>>(
        "edit_field/get_edit_fields",
        request,
      );
      return response;
    }),
    shareReplay(1),
  );
  return getEditFieldsCollection[args.objectKind][args.id];
}
