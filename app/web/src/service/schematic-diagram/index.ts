import { ApiResponse, sdf } from "@/api/sdf";
import {
  combineLatest,
  firstValueFrom,
  shareReplay,
  map,
  switchMap,
} from "rxjs";
import { applicationNodeId$ } from "@/observable/application";
import {
  standardVisibilityTriggers$,
  visibility$,
} from "@/observable/visibility";
import _ from "lodash";
import { DiagramContent } from "@/organisms/GenericDiagram/diagram_types";
import { system$ } from "@/observable/system";
import { useObservable } from "@vueuse/rxjs";
import { Vector2d } from "konva/lib/types";
import { NodePosition } from "@/api/sdf/dal/node_position";
import { GlobalErrorService } from "../global_error";
import { workspace$ } from "@/observable/workspace";
import {
  SchematicNode,
  SchematicSchemaVariants,
} from "@/api/sdf/dal/schematic";

export type GetSchematicResponse = DiagramContent;

const schematicDiagramData$ = combineLatest([
  system$,
  visibility$,
  standardVisibilityTriggers$,
  applicationNodeId$,
]).pipe(
  switchMap(([system, visibility, _visibilityTriggers, _applicationNodeId]) => {
    return sdf.get<ApiResponse<GetSchematicResponse>>(
      "schematic/get_schematic",
      {
        ...(system?.id && { systemId: system.id }),
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

function useDiagramData() {
  // TODO: maybe have this return something like { error, loading, data }?
  // not a huge fan of how the observable may return either an error OR the data we want
  // IMO it would make more sense to let the component grab them seperately so it doesn't have to check
  // by returning everything split, the consumer can choose to care or not care about the error (and loading state)
  return useObservable(schematicDiagramData$);
}

type ListSchemaVariantsResponse = SchematicSchemaVariants;
const schemaVariants$ = combineLatest([standardVisibilityTriggers$]).pipe(
  switchMap(([[visibility]]) => {
    return sdf.get<ApiResponse<ListSchemaVariantsResponse>>(
      "schematic/list_schema_variants",
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

//////////////////////////////////////////////////////////////////////////////////////////////

interface UpdateNodePositionResponse {
  position: NodePosition;
}

async function updateNodePosition(nodeId: string, position: Vector2d) {
  const visibility = await firstValueFrom(visibility$);
  const system = await firstValueFrom(system$);
  const req$ = sdf.post<ApiResponse<UpdateNodePositionResponse>>(
    "schematic/set_node_position",
    {
      schematicKind: "component",
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
  node: SchematicNode;
}

async function createNode(schemaId: number, position: Vector2d) {
  const visibility = await firstValueFrom(visibility$);
  const workspace = await firstValueFrom(workspace$);

  if (!workspace)
    throw new Error("Cannot select insert node without a workspace");

  const req$ = sdf.post<ApiResponse<CreateNodeResponse>>(
    "schematic/create_node",
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
    "schematic/create_connection",
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
