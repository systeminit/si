import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable, shareReplay } from "rxjs";
import { applicationNodeId$ } from "@/observable/application";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { Visibility } from "@/api/sdf/dal/visibility";
import { SchematicKind } from "@/api/sdf/dal/schematic";

// These datastructures should die after the backend starts returning SchematicNode

interface Socket {
  id: number;
  name: string;
}

interface NodePosition {
  x: number | string;
  y: number | string;
  schematic_kind: SchematicKind;
  deployment_node_id?: number;
  system_id?: number;
}

export interface Node {
  id: number;
  kind: { kind: "component" | "deployment"; componentId?: number };
  label: {
    name: string;
    title: string;
  };
  position: NodePosition[];
  input: Socket[];
  output: Socket[];
}

export interface GetSchematicArgs {
  systemId?: number;
}

export interface GetSchematicRequest extends GetSchematicArgs, Visibility {
  workspaceId: number;
}

interface Port {
  nodeId: number;
  socketId: number;
}

export interface Connection {
  id: number;
  source: Port;
  destination: Port;
}

// This datastructure should die after the backend starts returning Schematic
export interface GetSchematicResponse {
  nodes: Node[];
  connections: Connection[];
}

const getSchematicCollection: {
  [key: string]: Observable<ApiResponse<GetSchematicResponse>>;
} = {};

export function getSchematic(
  args: GetSchematicArgs,
): Observable<ApiResponse<GetSchematicResponse>> {
  const context = `${args.systemId}`;
  if (getSchematicCollection[context]) {
    return getSchematicCollection[context];
  }
  getSchematicCollection[context] = combineLatest([
    standardVisibilityTriggers$,
    workspace$,
    applicationNodeId$, // Application id is passed implicitly but we need the reactivity
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

      // TODO parse response to convert node position from string to number.
      const schematicResponse = sdf.get<ApiResponse<GetSchematicResponse>>(
        "schematic/get_schematic",
        {
          ...args,
          ...visibility,
          workspaceId: workspace.id,
        },
      );

      return schematicResponse;
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getSchematicCollection[context];
}
