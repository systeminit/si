import { StandardModel } from "@/api/sdf/dal/standard_model";
import { DiagramKind } from "@/api/sdf/dal/diagram";
import { CodeView } from "@/api/sdf/dal/code_view";
import { Resource } from "@/api/sdf/dal/resource";
import { Actor } from "@/store/status.store";

export interface Component extends StandardModel {
  name: string;
}

export interface ComponentIdentificationTimestamp {
  actor: Actor;
  timestamp: string;
}

export interface ComponentIdentification {
  componentId: string;
  schemaVariantId: string;
  schemaVariantName: string;
  schemaId: string;
  schemaName: string;
  diagramKind: DiagramKind;
  resource?: Resource;
  createdAt: ComponentIdentificationTimestamp;
  updatedAt: ComponentIdentificationTimestamp;
}

export interface ComponentDiff {
  current: CodeView;
  diffs: Array<CodeView>;
}
