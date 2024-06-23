import * as _ from "lodash-es";

export enum ComponentType {
  Component = "component",
  ConfigurationFrameDown = "configurationFrameDown",
  ConfigurationFrameUp = "configurationFrameUp",
  AggregationFrame = "aggregationFrame",
}

export type DiagramKind = "configuration";

export type OutputSocketId = string;

export interface DiagramOutputSocket {
  id: OutputSocketId;
  name: string;
}

export type InputSocketId = string;

export interface DiagramInputSocket {
  id: InputSocketId;
  name: string;
}

export interface DiagramSchemaVariant {
  id: string;
  name: string;
  builtin: boolean;
  isDefault: boolean;
  componentType: ComponentType;

  color: string;
  category: string;
  inputSockets: DiagramInputSocket[];
  outputSockets: DiagramOutputSocket[];
  created_at: IsoDateString;
  updated_at: IsoDateString;
  displayName: string | null;
  description: string | null;
}

export interface DiagramSchema {
  id: string;
  name: string; // schema name
  displayName: string; // variant display name
  builtin: boolean;

  variants: DiagramSchemaVariant[];
}

export interface DiagramNodeKindComponent {
  kind: DiagramKind;
  componentId: string;
}

export type DiagramNodeKind = DiagramNodeKindComponent;

export interface DiagramNodePosition {
  diagramKind: DiagramKind;
  x: number;
  y: number;
  width?: number;
  height?: number;
}

export interface DiagramNodeTemplate {
  name: string;
  title: string;
  kind: DiagramKind;
  schemaVariantId: string;
}

export interface DiagramNode {
  id: string;
  name: string;
  title: string;
  kind: DiagramNodeKind;
  schemaVariantId: string;
  positions: DiagramNodePosition[];
}

export type DiagramNodes = Array<DiagramNode>;
