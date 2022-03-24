import { Color, SchematicObject } from "./common";
import { Socket } from "./socket";
import { NodeTemplate } from "@/api/sdf/dal/node";
import { NodeKind } from "@/api/sdf/dal/node";

export interface NodeLabel {
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
export interface NodeClassification {
  component: ComponentType; // update to correct value
  kind: string; // update to correct value
  type: string; // update to correct value
}

/**  Node positions for a given context */
interface NodePosition {
  id: string;
  x: number | string;
  y: number | string;
  schematicKind?: SchematicLevel;
  rootNodeId?: string;
  applicationId?: string;
  systemId?: string;
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
  qualification?: QualificationStatus;
  resource?: ResourceStatus;
  changeCount?: number;
  action?: NodeAction;
}

/**  Display properties */
export interface NodeDisplay {
  color?: Color;
}

interface NodeConnection {
  id: string;
}

/**  A node */
export interface Node extends SchematicObject {
  id: number;
  kind: NodeKind;
  label: NodeLabel;
  classification: NodeClassification;
  status?: NodeStatus;
  position: NodePosition[]; // TODO: refactor schematic & schematicResponses
  input: Socket[];
  output: Socket[];
  connections?: NodeConnection[];
  display?: NodeDisplay;
}

export interface NodePositionUpdate {
  ctxId: string;
  x: number;
  y: number;
}

export interface NodeUpdate {
  nodeId: number;
  position: NodePositionUpdate;
}

export function fakeNodeFromTemplate(template: NodeTemplate): Node {
  const node: Node = {
    id: -1,
    kind: template.kind,
    label: template.label,
    classification: template.classification,
    position: [
      {
        id: "-1",
        x: 0,
        y: 0,
      },
    ],
    input: template.input,
    output: template.output,
    display: template.display,
    lastUpdated: new Date(Date.now()),
    checksum: "j4j4j4j4j4j4j4j4j4j4j4",
    schematic: {
      deployment: template.kind === NodeKind.Deployment,
      component: template.kind === NodeKind.Component,
    },
  };
  return node;
}

export function generateNodeName(): string {
  const name = "si-";
  return name;
}
