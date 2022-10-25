// This is a map of valid websocket events to the shape of their payload
// used in the subscribe fn to limit valid event names and set callback payload type

// TODO: a few of these use the same id objects (ex: componentId + systemId)
// but in a few cases the changeset ID may have been accidentally left out?
// once things are working again, we should do a big review of all the realtime events coming from the backend...

export type WsEventPayloadMap = {
  ChangeSetCreated: number;
  ChangeSetApplied: number;
  ChangeSetWritten: number;
  ChangeSetCancelled: number;

  CheckedQualifications: {
    prototypeId: number;
    componentId: number;
    systemId: number;
  };

  CodeGenerated: {
    componentId: number;
    systemId: number;
  };

  ConfirmationStatusUpdate: {
    componentId: number;
    systemId: number;
    confirmationPrototypeId: number;
    status: "success" | "running" | "pending" | "failure" | "error"
    errorMessage?: string;
  };

  // NOT CURRENTLY USED - but leaving here so we remember these events exist
  // SecretCreated: number;
  // ResourceRefreshed: {
  //   componentId: number;
  //   systemId: number;
  // }
  // UpdatedDependentValue: {
  //   componentId: number;
  //   systemId: number;
  // }
  // CommandOutput: { runId: number; output: string }
  // CommandReturn: {
  //   runId: number;
  //   createdResources: Resource[];
  //   updatedResources: Resource[];
  //   output: string[];
  //   runnerState: WorkflowRunnerState;
  // };
  FixReturn: {
    confirmationResolverId: number;
    output: string[];
    runnerState: { status: "failure" | "success" | "running" | "created" },
  }
};
