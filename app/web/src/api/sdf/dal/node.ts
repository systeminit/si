import { Socket } from "@/organisims/SchematicViewer/model/socket";
import {
  NodeLabel,
  NodeClassification,
  NodeDisplay,
} from "@/organisims/SchematicViewer/model/node";

export enum NodeKind {
  Deployment = "deployment",
  Component = "component",
}

export interface NodeTemplate {
  kind: NodeKind;
  label: NodeLabel;
  classification: NodeClassification;
  input: Socket[];
  output: Socket[];
  display?: NodeDisplay;
}
