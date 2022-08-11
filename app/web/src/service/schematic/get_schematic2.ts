import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, shareReplay } from "rxjs";
import { applicationNodeId$ } from "@/observable/application";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import _ from "lodash";
import { Visibility } from "@/api/sdf/dal/visibility";
import { DiagramSchematicDef } from "./diagram_types";

export interface GetSchematicArgs {
  systemId?: number;
}

export interface GetSchematicRequest extends GetSchematicArgs, Visibility {}

export type GetSchematicResponse = DiagramSchematicDef;

const getSchematicCollection: {
  [key: string]: Observable<ApiResponse<GetSchematicResponse>>;
} = {};

export function getSchematic2(
  args: GetSchematicArgs,
): Observable<ApiResponse<GetSchematicResponse>> {
  const context = `${args.systemId}`;
  if (getSchematicCollection[context]) {
    return getSchematicCollection[context];
  }
  getSchematicCollection[context] = combineLatest([
    standardVisibilityTriggers$,
    applicationNodeId$, // Application id is passed implicitly but we need the reactivity
  ]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;

      const schematicResponse = sdf.get<ApiResponse<GetSchematicResponse>>(
        "schematic/get_schematic2",
        {
          ...args,
          ...visibility,
        },
      );

      return schematicResponse;
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getSchematicCollection[context];
}
