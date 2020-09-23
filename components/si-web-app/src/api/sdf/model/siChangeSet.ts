import { IUpdateClock } from "./updateClock";

export enum SiChangeSetEvent {
  Create = "Create",
  Delete = "Delete",
  Operation = "Operation",
  Action = "Action",
}

export interface ISiChangeSet {
  change_set_id: string;
  edit_session_id: string;
  event: SiChangeSetEvent;
  order_clock: IUpdateClock;
}
