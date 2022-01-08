import { ApiResponse, SDF } from "@/api/sdf";
import { Schematic } from "@/api/sdf/dal/schematic";
import { combineLatest, from, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { workspace$ } from "@/observable/workspace";
import { system$ } from "@/observable/system";
import { application$ } from "@/observable/application";
import _ from "lodash";

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
  if (getSchematicCollection[args.context]) {
    return getSchematicCollection[args.context];
  }
  getSchematicCollection[args.context] = combineLatest([
    standardVisibilityTriggers$,
    workspace$,
    system$,
    application$,
  ]).pipe(
    switchMap(([[visibility], workspace, system, application]) => {
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
      if (_.isNull(application)) {
        return from([
          {
            error: {
              statusCode: 10,
              message: "cannot get schematic without an application; bug!",
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
	  systemId: system?.id,
	  rootNodeId: application.id,
          workspaceId: workspace.id,
        },
      );
    }),
    shareReplay(1),
  );
  return getSchematicCollection[args.context];
}
