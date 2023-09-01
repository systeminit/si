// This is a map of valid websocket events to the shape of their payload
// used in the subscribe fn to limit valid event names and set callback payload type

import { ActorView } from "@/api/sdf/dal/history_actor";
import { ComponentId } from "../components.store";
import { FixStatus } from "../fixes.store";
import {
  AttributeValueId,
  AttributeValueKind,
  AttributeValueStatus,
  StatusUpdatePk,
} from "../status.store";

// TODO: a few of these use the same id objects (ex: componentId)
// but in a few cases the changeset ID may have been accidentally left out?
// once things are working again, we should do a big review of all the realtime events coming from the backend...

export type WsEventPayloadMap = {
  ChangeSetCreated: string;
  ChangeSetApplied: string;
  ChangeSetWritten: string;
  ChangeSetCancelled: string;

  CheckedQualifications: {
    prototypeId: string;
    componentId: string;
  };

  CodeGenerated: {
    componentId: string;
  };

  // NOT CURRENTLY USED - but leaving here so we remember these events exist
  // SecretCreated: number;
  ResourceRefreshed: {
    componentId: string;
  };
  // UpdatedDependentValue: {
  //   componentId: string;
  // }
  // CommandOutput: { runId: string; output: string }
  // CommandReturn: {
  //   runId: string;
  //   resources: Resource[];
  //   output: string[];
  //   runnerState: WorkflowRunnerState;
  // };

  FixReturn: {
    id: string;
    batchId: string;
    attributeValueId: string;
    action: string;
    output: string[];
    status: FixStatus;
  };
  FixBatchReturn: {
    id: string;
    status: FixStatus;
  };
  ComponentCreated: {
    success: boolean;
  };

  // Old fake status update
  // UpdateStatus: {
  //   global: GlobalUpdateStatus;
  //   components?: ComponentUpdateStatus[];
  // };

  StatusUpdate: {
    pk: StatusUpdatePk;
    status: AttributeValueStatus | "statusStarted" | "statusFinished";
    actor: ActorView;
    values: {
      componentId: ComponentId;
      valueId: AttributeValueId;
      valueKind: {
        kind: AttributeValueKind;
        id?: string;
      };
    }[];
  };
};
