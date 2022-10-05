// This is a map of valid websocket events to the shape of their payload
// used in the subscribe fn to limit valid event names and set callback payload type
export type WsEventPayloadMap = {
  ChangeSetCreated: number;
  ChangeSetApplied: number;
  ChangeSetWritten: number;
  ChangeSetCancelled: number;

  CheckedQualifications: CheckedQualificationId;
};

interface CheckedQualificationId {
  prototypeId: number;
  componentId: number;
  systemId: number;
}
