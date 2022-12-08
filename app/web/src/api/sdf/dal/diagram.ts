import _ from "lodash";

export type DiagramKind = "configuration";

export type DiagramProviderMetadata = string;

export interface DiagramOutputProvider {
  id: string;
  ty: DiagramProviderMetadata;
  color: number;
}

export interface DiagramOutputSocket {
  id: string;
  name: string;
  diagramKind: DiagramKind;
  provider: DiagramOutputProvider;
}

export interface DiagramInputProvider {
  id: string;
  ty: DiagramProviderMetadata;
  color: number;
}

export interface DiagramInputSocket {
  id: string;
  name: string;
  diagramKind: DiagramKind;
  provider: DiagramInputProvider;
}

export interface DiagramSchemaVariant {
  id: string;
  name: string;
  schemaName: string;
  color: number;
  inputSockets: DiagramInputSocket[];
  outputSockets: DiagramOutputSocket[];
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

export type DiagramSchemaVariants = Array<DiagramSchemaVariant>;
