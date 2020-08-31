import { IUpdateClock } from "./updateClock";

export enum SiChangeSetEvent {
  Create = "Create",
  Delete = "Delete",
  Operation = "Operation",
  Action = "Action",
}

export interface ISiChangeSet {
  changeSetId: string;
  editSessionId: string;
  event: SiChangeSetEvent;
  order_clock: IUpdateClock;
}
