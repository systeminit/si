import * as _ from "lodash-es";

export type DiagramKind = "configuration";

export interface DiagramOutputSocket {
  id: string;
  name: string;
}

export interface DiagramInputSocket {
  id: string;
  name: string;
}

export interface DiagramSchemaVariant {
  id: string;
  name: string;
  builtin: boolean;

  color: string;
  category: string;
  inputSockets: DiagramInputSocket[];
  outputSockets: DiagramOutputSocket[];
}

export interface DiagramSchema {
  id: string;
  name: string;
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
