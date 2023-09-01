import { ComponentId } from "@/store/components.store";

export enum ChangeSetStatus {
  Open = "Open",
  Closed = "Closed",
  Abandoned = "Abandoned",
  Applied = "Applied",
  Failed = "Failed",
}

export type ActionPrototypeId = string;
export interface ActionPrototype {
  id: ActionPrototypeId;
  name: string;
}

export interface NewAction {
  id: never;
  prototypeId: ActionPrototypeId;
  name: string;
  componentId: ComponentId;
}

export type ActionId = string;
export interface Action {
  id: ActionId;
  name: string;
  componentId: ComponentId;
}

export type ChangeSetId = string;
export interface ChangeSet {
  id: ChangeSetId;
  pk: ChangeSetId;
  name: string;
  actions: Action[];
  status: ChangeSetStatus;
}

export type ChangeStatus = "added" | "deleted" | "modified" | "unmodified";

export interface ComponentStatsGroup {
  componentId: string;
  componentName: string;
  componentStatus: ChangeStatus;
}

export interface ComponentStats {
  stats: ComponentStatsGroup[];
}
