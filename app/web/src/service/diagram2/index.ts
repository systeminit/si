// FIXME(nick): schematics have been renamed to diagrams. As a result, this service was renamed
// from "schematic-diagram" to "diagram2". Both diagram services should be combined, but that
// was out of scope for the backend refactor PR of removing applications, services, deployment
// nodes, etc.

import {
  combineLatest,
  firstValueFrom,
  shareReplay,
  map,
  switchMap,
} from "rxjs";
import _ from "lodash";
import { useObservable } from "@vueuse/rxjs";
import { Vector2d } from "konva/lib/types";
import { ApiResponse, sdf } from "@/api/sdf";
import {
  standardVisibilityTriggers$,
  visibility$,
} from "@/observable/visibility";
import { DiagramContent } from "@/organisms/GenericDiagram/diagram_types";
import { system$ } from "@/observable/system";
import { NodePosition } from "@/api/sdf/dal/node_position";
import { workspace$ } from "@/observable/workspace";
import { DiagramNode, DiagramSchemaVariants } from "@/api/sdf/dal/diagram";
import { GlobalErrorService } from "../global_error";

export type GetDiagramResponse = DiagramContent;

const diagramDiagramData$ = combineLatest([
  system$,
  visibility$,
  standardVisibilityTriggers$,
]).pipe(
  switchMap(([system, visibility, _visibilityTriggers]) => {
    return sdf.get<ApiResponse<GetDiagramResponse>>("diagram/get_diagram", {
      ...(system?.id && { systemId: system.id }),
      ...visibility,
    });
  }),
  map((apiResponse) => {
    if (apiResponse.error) {
      GlobalErrorService.set(apiResponse);
      return null;
    }
    return apiResponse;
  }),
  shareReplay({ bufferSize: 1, refCount: true }),
);

function useDiagramData() {
  // TODO: maybe have this return something like { error, loading, data }?
  // not a huge fan of how the observable may return either an error OR the data we want
  // IMO it would make more sense to let the component grab them seperately so it doesn't have to check
  // by returning everything split, the consumer can choose to care or not care about the error (and loading state)
  return useObservable(diagramDiagramData$);
}

type ListSchemaVariantsResponse = DiagramSchemaVariants;
const schemaVariants$ = combineLatest([standardVisibilityTriggers$]).pipe(
  switchMap(([[visibility]]) => {
    return sdf.get<ApiResponse<ListSchemaVariantsResponse>>(
      "diagram/list_schema_variants",
      {
        ...visibility,
      },
    );
  }),
  map((apiResponse) => {
    if (apiResponse.error) {
      GlobalErrorService.set(apiResponse);
      return null;
    }
    return apiResponse;
  }),
  shareReplay({ bufferSize: 1, refCount: true }),
);

function useSchemaVariants() {
  return useObservable(schemaVariants$);
}

// ---------------------------------------------------

interface UpdateNodePositionResponse {
  position: NodePosition;
}

async function updateNodePosition(nodeId: string, position: Vector2d) {
  const visibility = await firstValueFrom(visibility$);
  const system = await firstValueFrom(system$);
  const req$ = sdf.post<ApiResponse<UpdateNodePositionResponse>>(
    "diagram/set_node_position",
    {
      diagramKind: "configuration",
      x: position.x.toString(),
      y: position.y.toString(),
      nodeId: parseInt(nodeId),
      systemId: system?.id,
      ...visibility,
    },
  );
  const response = await firstValueFrom(req$);
  if (response.error) {
    GlobalErrorService.set(response);
    return;
  }
  // editSessionWritten$.next(true);
  return response;
}

export interface CreateNodeResponse {
  node: DiagramNode;
}

async function createNode(schemaId: number, position: Vector2d) {
  const visibility = await firstValueFrom(visibility$);
  const workspace = await firstValueFrom(workspace$);

  if (!workspace)
    throw new Error("Cannot select insert node without a workspace");

  const req$ = sdf.post<ApiResponse<CreateNodeResponse>>(
    "diagram/create_node",
    {
      schemaId,
      x: position.x.toString(),
      y: position.y.toString(),
      ...visibility,
      workspaceId: workspace.id,
    },
  );
  const response = await firstValueFrom(req$);
  if (response.error) {
    GlobalErrorService.set(response);
    return;
  }
  // editSessionWritten$.next(true);
  return response;
}

export type CreateConnectionArgs = {
  fromNodeId: string;
  fromSocketId: string;
  toNodeId: string;
  toSocketId: string;
};

async function createConnection(args: CreateConnectionArgs) {
  const visibility = await firstValueFrom(visibility$);
  const workspace = await firstValueFrom(workspace$);

  if (!workspace) throw new Error("Cannot connect nodes without a workspace");

  const req$ = sdf.post<ApiResponse<CreateNodeResponse>>(
    "diagram/create_connection",
    {
      fromNodeId: parseInt(args.fromNodeId),
      fromSocketId: parseInt(args.fromSocketId),
      toNodeId: parseInt(args.toNodeId),
      toSocketId: parseInt(args.toSocketId),
      workspaceId: workspace.id,
      ...visibility,
    },
  );
  const response = await firstValueFrom(req$);
  if (response.error) {
    GlobalErrorService.set(response);
    return;
  }
  // editSessionWritten$.next(true);
  return response;
}

export default {
  useDiagramData,
  useSchemaVariants,
  observables: {
    schemaVariants$,
  },
  actions: {
    updateNodePosition,
    createNode,
    createConnection,
  },
};
