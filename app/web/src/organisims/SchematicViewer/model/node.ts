import { Color, SchematicObject } from "./common";
import { Socket } from "./socket";
import { NodeTemplate } from "@/api/sdf/dal/node";
import { SchematicKind } from "@/api/sdf/dal/schematic";

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
  id: string;
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
  nodeId: string;
  position: NodePositionUpdate;
}

export function fakeNodeFromTemplate(template: NodeTemplate): Node {
  const node: Node = {
    id: -1,
    label: template.label,
    classification: template.classification,
    position: [
      {
        id: -1,
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
      deployment: template.kind === SchematicKind.Deployment,
      component: template.kind === SchematicKind.Component,
    },
  };
  return node;
}

export function generateNodeName(): string {
  const name = "si-";
  return name;
}

export function generateNode(
  id: string,
  title: string,
  name: string,
  position: { x: number; y: number },
  schematicKind: SchematicKind,
): Node {
  const node: Node = {
    id: -1,
    label: {
      title: title,
      name: name,
    },
    classification: {
      component: ComponentType.APPLICATION,
      kind: "kubernetes",
      type: "service",
    },
    position: [
      {
        id,
        x: position.x,
        y: position.y,
      },
    ],
    input: [
      {
        id: -2,
        type: "kubernetes.namespace",
        name: "namespace",
      },
      {
        id: -3,
        type: "kubernetes.deployment",
        name: "deployment",
      },
      {
        id: -4,
        type: "kubernetes.service",
        name: "service",
      },
      {
        id: -5,
        type: "kubernetes.env",
        name: "env",
      },
    ],
    output: [
      {
        id: -6,
        type: "kubernetes.service",
      },
    ],
    display: {
      color: 0x32b832,
    },
    lastUpdated: new Date(Date.now()),
    checksum: "j4j4j4j4j4j4j4j4j4j4j4",
    schematic: {
      deployment: schematicKind === SchematicKind.Deployment,
      component: schematicKind === SchematicKind.Component,
    },
  };

  return node;
}
