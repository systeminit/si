import { IRect, Vector2d } from "konva/lib/types";
import { StandardModel } from "@/api/sdf/dal/standard_model";
import { CodeView } from "@/api/sdf/dal/code_view";
import { ActorView } from "@/api/sdf/dal/history_actor";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { ComponentType } from "@/api/sdf/dal/schema";
import { ViewDescription, ViewId } from "@/api/sdf/dal/views";
import {
  DiagramSocketDef,
  DiagramSocketDirection,
  Size2D,
} from "@/components/ModelingDiagram/diagram_types";

export interface Component extends StandardModel {
  name: string;
}

export interface ComponentIdentificationTimestamp {
  actor: ActorView;
  timestamp: string;
}

export interface ComponentDiff {
  componentId: string;
  current: CodeView;
  diff?: CodeView;
}

export interface ActorAndTimestamp {
  actor: ActorView;
  timestamp: string;
}

export type ComponentId = string;

export interface ViewGeometry {
  viewId: ViewId;
  geometry: Vector2d & Partial<Size2D>;
}

export interface ViewNodeGeometry {
  view: ViewDescription;
  geometry: IRect;
}

export interface RawComponent {
  changeStatus: ChangeStatus;
  color: string;
  createdInfo: ActorAndTimestamp;
  deletedInfo?: ActorAndTimestamp;
  displayName: string;
  resourceId: string;
  id: ComponentId;
  componentType: ComponentType;
  parentId?: ComponentId;
  hasResource: boolean;
  schemaCategory: string;
  schemaId: string; // TODO: probably want to move this to a different store and not load it all the time
  schemaName: string;
  schemaVariantId: string;
  schemaVariantName: string;
  schemaDocsLink?: string;
  sockets: DiagramSocketDef[];
  updatedInfo: ActorAndTimestamp;
  toDelete: boolean;
  canBeUpgraded: boolean;
  fromBaseChangeSet: boolean;
  viewData?: ViewGeometry;
}

export type EdgeId = string;
export type SocketId = string;

export type RawEdge = {
  fromComponentId: ComponentId;
  fromSocketId: SocketId;
  toComponentId: ComponentId;
  toSocketId: SocketId;
  toDelete: boolean;
  /** change status of edge in relation to head */
  changeStatus?: ChangeStatus;
  createdInfo: ActorAndTimestamp;
  // updatedInfo?: ActorAndTimestamp; // currently we dont ever update an edge...
  deletedInfo?: ActorAndTimestamp;
};

export type Edge = RawEdge & {
  id: EdgeId;
  isInferred: boolean;
  isManagement?: boolean;
};

export interface PotentialConnection {
  socketId: SocketId;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  value: any | null;
  attributeValueId: string;
  direction: DiagramSocketDirection;
  matches: PotentialMatch[];
}
export interface PotentialMatch {
  socketId: SocketId;
  componentId: ComponentId;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  value: any | null;
}
