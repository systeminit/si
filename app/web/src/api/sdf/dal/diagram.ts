import _ from "lodash";

export type DiagramKind = "configuration";

export type DiagramProviderMetadata = string;

export interface DiagramOutputProvider {
  id: number;
  ty: DiagramProviderMetadata;
  color: number;
}

export interface DiagramOutputSocket {
  id: number;
  name: string;
  diagramKind: DiagramKind;
  provider: DiagramOutputProvider;
}

export interface DiagramInputProvider {
  id: number;
  ty: DiagramProviderMetadata;
  color: number;
}

export interface DiagramInputSocket {
  id: number;
  name: string;
  diagramKind: DiagramKind;
  provider: DiagramInputProvider;
}

export interface DiagramSchemaVariant {
  id: number;
  name: string;
  schemaName: string;
  color: number;
  inputSockets: DiagramInputSocket[];
  outputSockets: DiagramOutputSocket[];
}

export interface DiagramNodeKindComponent {
  kind: DiagramKind;
  componentId: number;
}

export type DiagramNodeKind = DiagramNodeKindComponent;

export interface DiagramNodePosition {
  diagramKind: DiagramKind;
  x: number;
  y: number;
}

export interface DiagramNodeTemplate {
  name: string;
  title: string;
  kind: DiagramKind;
  schemaVariantId: number;
}

export interface DiagramNode {
  id: number;
  name: string;
  title: string;
  kind: DiagramNodeKind;
  schemaVariantId: number;
  positions: DiagramNodePosition[];
}
export type DiagramNodes = Array<DiagramNode>;

export type DiagramSchemaVariants = Array<DiagramSchemaVariant>;
