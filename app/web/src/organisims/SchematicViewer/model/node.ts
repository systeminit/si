import { Color, SchematicObject, Position } from "./common";
import { Socket } from "./socket";

interface NodeLabel {
  title: string;
  name: string;
}

export enum ComponentType {
  APPLICATION = "application",
  COMPUTING = "computing",
}

export enum SchematicLevel {
  APPLICATION = "deployment",
  COMPUTING = "component",
}

/**
 * Node classification
 * @kind application component | computing component | provider component | ...
 * @type Service | AWS | Kubernetes | Implementation | ...
 * @subtype AWS.region | Kubernetes.service | Implementation.kubernetes | ...
 *
 * This will evolve over time as we grow the list of component
 * */
interface NodeClassification {
  component: ComponentType; // update to correct value
  kind: string; // update to correct value
  type: string; // update to correct value
}

/**  Node positions for a given context */
interface NodePositionContext {
  id: string;
  schematicKind?: SchematicLevel;
  rootNodeId?: string;
  applicationId?: string;
  systemId?: string;
  position: Position;
}

/**  Node position for contextes */
interface NodePosition {
  ctx: NodePositionContext[];
}

export enum QualificationStatus {
  SUCCEEDED = "succeeded",
  FAILED = "failed",
}

export enum ResourceStatus {
  HEALTHY = "healthy",
  UNHEALTHY = "unhealthy",
}

export enum ActionStatus {
  PENDING = "pending",
  RUNNING = "running",
  SUCCEEDED = "succeeded",
  FAILED = "failed",
}

/** Latest node action */
interface NodeAction {
  name: string;
  timestamp: Date;
  status: ActionStatus;
}

/**
 * Statuses
 * @qualification
 * @resource
 * @action
 * @changeCount
 * */
interface NodeStatus {
  qualification: QualificationStatus;
  resource: ResourceStatus;
  changeCount: number;
  action?: NodeAction;
}

/**  Display properties */
interface NodeDisplay {
  color?: Color;
}

interface NodeConnection {
  id: string;
}

/**  A node */
export interface Node extends SchematicObject {
  id: string;
  label: NodeLabel;
  classification: NodeClassification;
  status: NodeStatus;
  position: NodePosition;
  input: Socket[];
  output: Socket[];
  connections: NodeConnection[];
  display?: NodeDisplay;
}

export interface NodePositionUpdate {
  ctxId: string;
  x: number;
  y: number;
}

export interface NodeUpdate {
  nodeId: string;
  position: NodePositionUpdate;
}
