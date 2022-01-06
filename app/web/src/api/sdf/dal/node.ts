import { Socket } from "@/organisims/SchematicViewer/model/socket";
import {
  NodeLabel,
  NodeClassification,
  NodeDisplay,
} from "@/organisims/SchematicViewer/model/node";

export interface NodeTemplate {
  label: NodeLabel;
  classification: NodeClassification;
  input: Socket[];
  output: Socket[];
  display?: NodeDisplay;
}
