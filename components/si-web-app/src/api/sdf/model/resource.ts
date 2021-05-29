import { ISiStorable } from "@/api/sdf/model/siStorable";
import _ from "lodash";
import Bottle from "bottlejs";

export enum ResourceHealth {
  Ok = "ok",
  Warning = "warning",
  Error = "error",
  Unknown = "unknown",
}

export enum ResourceStatus {
  Pending = "pending",
  InProgress = "inProgress",
  Created = "created",
  Failed = "failed",
  Deleted = "deleted",
}

export interface IResource {
  id: string;
  unixTimestamp: string;
  timestamp: string;
  state: any;
  status: ResourceStatus;
  health: ResourceHealth;
  systemId: string;
  nodeId: string;
  entityId: string;
  entityType: string;
  siStorable: ISiStorable;
}

export class Resource implements IResource {
  id: IResource["id"];
  unixTimestamp: IResource["unixTimestamp"];
  timestamp: IResource["timestamp"];
  state: IResource["state"];
  status: IResource["status"];
  health: IResource["health"];
  systemId: IResource["systemId"];
  nodeId: IResource["nodeId"];
  entityId: IResource["entityId"];
  entityType: string;
  siStorable: IResource["siStorable"];

  constructor(args: IResource) {
    this.id = args.id;
    this.unixTimestamp = args.unixTimestamp;
    this.timestamp = args.timestamp;
    this.state = args.state;
    this.status = args.status;
    this.health = args.health;
    this.systemId = args.systemId;
    this.nodeId = args.nodeId;
    this.entityId = args.entityId;
    this.siStorable = args.siStorable;
    this.entityType = args.entityType;
  }

  static upgrade(obj: Resource | IResource): Resource {
    if (obj instanceof Resource) {
      return obj;
    } else {
      return new Resource(obj);
    }
  }
}
