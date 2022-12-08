import { StandardModel } from "@/api/sdf/dal/standard_model";
import { DiagramKind } from "@/api/sdf/dal/diagram";
import { CodeView } from "@/api/sdf/dal/code_view";
import { Resource } from "@/api/sdf/dal/resource";

export interface Component extends StandardModel {
  name: string;
}

export interface ComponentIdentification {
  componentId: string;
  schemaVariantId: string;
  schemaVariantName: string;
  schemaId: string;
  schemaName: string;
  diagramKind: DiagramKind;
  resource?: Resource;
}

export interface ComponentDiff {
  current: CodeView;
  diffs: Array<CodeView>;
}
