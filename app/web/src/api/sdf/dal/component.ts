import { IRect, Vector2d } from "konva/lib/types";
import { StandardModel } from "@/api/sdf/dal/standard_model";
import { CodeView } from "@/api/sdf/dal/code_view";
import { ActorView } from "@/api/sdf/dal/history_actor";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import {
  ComponentType,
  InputSocketId,
  OutputSocketId,
} from "@/api/sdf/dal/schema";
import { ViewDescription, ViewId } from "@/api/sdf/dal/views";
import {
  DiagramSocketDef,
  DiagramSocketDirection,
  Size2D,
} from "@/components/ModelingDiagram/diagram_types";
import { TopLevelProp } from "./prop";

export interface Component extends StandardModel {
  name: string;
}

export interface ComponentIdentificationTimestamp {
  actor: ActorView;
  timestamp: string;
}

export interface ComponentTextDiff {
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
  parentId?: undefined;
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

export interface RawComponentEdge {
  fromComponentId: ComponentId;

  toComponentId: ComponentId;

  toDelete?: boolean;
  /** change status of edge in relation to head */
  changeStatus?: ChangeStatus;
  createdInfo?: ActorAndTimestamp;
  // updatedInfo?: ActorAndTimestamp; // currently we dont ever update an edge...
  deletedInfo?: ActorAndTimestamp;
}
export interface RawSocketEdge extends RawComponentEdge {
  fromSocketId: OutputSocketId;
  toSocketId: InputSocketId;
}
export interface RawSubscriptionEdge extends RawComponentEdge {
  fromAttributePath: AttributePath;
  toAttributePath: AttributePath;
}
export type RawEdge = RawSocketEdge | RawSubscriptionEdge;

export function isRawSocketEdge(edge: RawEdge): edge is RawSocketEdge {
  return "fromSocketId" in edge;
}
export function isRawSubscriptionEdge(
  edge: RawEdge,
): edge is RawSubscriptionEdge {
  return "fromAttributePath" in edge;
}

interface ExtraEdgeProperties {
  id: EdgeId;
  isInferred: boolean;
  isManagement?: boolean;
}
export interface SocketEdge extends RawSocketEdge, ExtraEdgeProperties {}
export interface SubscriptionEdge
  extends RawSubscriptionEdge,
    ExtraEdgeProperties {}
export type Edge = SocketEdge | SubscriptionEdge;
export function isSocketEdge(edge: Edge | undefined): edge is SocketEdge {
  return edge ? "fromSocketId" in edge : false;
}
export function isSubscriptionEdge(
  edge: Edge | undefined,
): edge is SubscriptionEdge {
  return edge ? "fromAttributePath" in edge : false;
}

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

/** JSON pointer to an attribute, relative to the component root (e.g. /domain/IpAddresses/0 or /si/name) */
// NOTE: This three-alternative type is used to ensure it is either the root (/), or a path under
// domain/resource/si, etc. Specifying it this way gives us nice autocompletions for "/domain"
// and friends under IDEs, too.
export type AttributePath =
  | "" // root
  | "/" // havent seen this in data yet
  | `/${TopLevelProp}`
  | `/${TopLevelProp}/${string}`;
