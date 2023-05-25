import { StandardModel } from "@/api/sdf/dal/standard_model";
import { CodeView } from "@/api/sdf/dal/code_view";
import { ActorView } from "@/api/sdf/dal/history_actor";

export interface Component extends StandardModel {
  name: string;
}

export interface ComponentIdentificationTimestamp {
  actor: ActorView;
  timestamp: string;
}

export interface ComponentDiff {
  componentId: string;
  current: CodeView;
  diffs: Array<CodeView>;
}
