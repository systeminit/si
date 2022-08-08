import { StandardModel } from "@/api/sdf/dal/standard_model";
import { SchematicKind } from "@/api/sdf/dal/schematic";
import { CodeView } from "@/api/sdf/dal/code_view";

export interface Component extends StandardModel {
  name: string;
}

export interface ComponentIdentification {
  componentId: number;
  schemaVariantId: number;
  schemaVariantName: string;
  schemaId: number;
  schemaName: string;
  schematicKind: SchematicKind;
}

export interface ComponentDiff {
  current: CodeView;
  diffs: Array<CodeView>;
}
