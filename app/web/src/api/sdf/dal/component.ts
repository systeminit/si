import { StandardModel } from "@/api/sdf/dal/standard_model";
import { CodeView } from "@/api/sdf/dal/code_view";
import { ActorView } from "@/api/sdf/dal/history_actor";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { ComponentType } from "@/api/sdf/dal/schema";
import {
  DiagramSocketDef,
  GridPoint,
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
  diffs: Array<CodeView>;
}

export interface ActorAndTimestamp {
  actor: ActorView;
  timestamp: string;
}

export type ComponentId = string;

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
  position: GridPoint;
  size?: Size2D;
  hasResource: boolean;
  schemaCategory: string;
  schemaId: string; // TODO: probably want to move this to a different store and not load it all the time
  schemaName: string;
  schemaVariantId: string;
  schemaVariantName: string;
  sockets: DiagramSocketDef[];
  updatedInfo: ActorAndTimestamp;
  toDelete: boolean;
  canBeUpgraded: boolean;
  fromBaseChangeSet: boolean;
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
};
