import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import { sdf } from "@/api/sdf";
import _ from "lodash";
import store from "@/store";

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
  }

  static async getByEntityIdAndSystemId(
    nodeId: string,
    systemId: string,
  ): Promise<Resource | undefined> {
    let iResults = await db.resources.where({ systemId, nodeId }).toArray();
    let results = _.map(iResults, ir => {
      return new Resource(ir);
    });
    if (results) {
      return results[0];
    } else {
      return undefined;
    }
  }

  async save(): Promise<void> {
    const currentObj = await db.resources.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.resources.put(this);
      await this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    await store.dispatch("editor/fromResource", this);
  }

  static async restore(): Promise<void> {
    let iObjects = await db.resources.toArray();
    for (const iobj of iObjects) {
      let obj = new Resource(iobj);
      await obj.dispatch();
    }
  }
}

db.resources.mapToClass(Resource);
