import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";
import { PropertyEditorSchema } from "@/api/sdf/dal/property_editor";

export interface GetPropertyEditorSchemaArgs {
  componentId: number;
}

export interface GetPropertyEditorSchemaRequest
  extends GetPropertyEditorSchemaArgs,
    Visibility {}

export type GetPropertyEditorSchemaResponse = PropertyEditorSchema;

const getPropertyEditorSchemaCollection: {
  [key: number]: Observable<ApiResponse<GetPropertyEditorSchemaResponse>>;
} = {};

export function getPropertyEditorSchema(
  args: GetPropertyEditorSchemaArgs,
): Observable<ApiResponse<GetPropertyEditorSchemaResponse>> {
  // NOTE:  This should eventually cache on schema id, and this interface needs to change.
  if (getPropertyEditorSchemaCollection[args.componentId]) {
    return getPropertyEditorSchemaCollection[args.componentId];
  }
  getPropertyEditorSchemaCollection[args.componentId] = combineLatest([
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.get<ApiResponse<GetPropertyEditorSchemaResponse>>(
        "component/get_property_editor_schema",
        {
          ...args,
          ...visibility,
        },
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getPropertyEditorSchemaCollection[args.componentId];
}
