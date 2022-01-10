import { ApiResponse, SDF } from "@/api/sdf";
import { Schematic } from "@/organisims/SchematicViewer/model/schematic";
import { combineLatest, from, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { Visibility } from "@/api/sdf/dal/visibility";

export interface GetSchematicArgs {
  systemId: number;
  rootNodeId: number;
}

export interface GetSchematicRequest extends GetSchematicArgs, Visibility {
  workspaceId: number;
}

export type GetSchematicResponse = Schematic;

const getSchematicCollection: {
  [key: string]: Observable<ApiResponse<Schematic>>;
} = {};

export function getSchematic(
  args: GetSchematicArgs,
): Observable<ApiResponse<GetSchematicResponse>> {
  const context = `${args.rootNodeId}-${args.systemId}`;
  if (getSchematicCollection[context]) {
    return getSchematicCollection[context];
  }
  getSchematicCollection[context] = combineLatest([
    standardVisibilityTriggers$,
    workspace$,
  ]).pipe(
    switchMap(([[visibility], workspace]) => {
      if (_.isNull(workspace)) {
        return from([
          {
            error: {
              statusCode: 10,
              message: "cannot get schematic without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.get<ApiResponse<GetSchematicResponse>>(
        "schematic/get_schematic",
        {
          ...args,
          ...visibility,
          systemId: args.systemId,
          rootNodeId: args.rootNodeId,
          workspaceId: workspace.id,
        },
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getSchematicCollection[context];
}
