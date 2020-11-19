import _ from "lodash";
import { DateTime } from "luxon";

import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import store from "@/store";
import { User } from "./user";

export enum EventStatus {
  Unknown = "unknown",
  Running = "running",
  Success = "success",
  Error = "error",
}

export enum EventKind {
  ResourceSync = "resourceSync",
  NodeEntityCreate = "nodeEntityCreate",
}

export interface IEvent {
  id: string;
  message: string;
  kind: EventKind;
  context: string[];
  payload: any;
  status: EventStatus;
  parentId?: string;
  startUnixTimestamp: number;
  startTimestamp: string;
  endUnixTimestamp: number;
  endTimestamp: string;
  siStorable: ISiStorable;
  owner?: string;
  name?: string;
}

export class Event implements IEvent {
  id: IEvent["id"];
  message: IEvent["message"];
  kind: IEvent["kind"];
  context: IEvent["context"];
  payload: IEvent["payload"];
  status: IEvent["status"];
  parentId?: IEvent["parentId"];
  startUnixTimestamp: IEvent["startUnixTimestamp"];
  startTimestamp: IEvent["startTimestamp"];
  endUnixTimestamp: IEvent["endUnixTimestamp"];
  endTimestamp: IEvent["endTimestamp"];
  siStorable: IEvent["siStorable"];
  owner?: IEvent["owner"];
  name: IEvent["name"];

  constructor(args: IEvent) {
    this.id = args.id;
    this.message = args.message;
    this.kind = args.kind;
    this.context = args.context;
    this.payload = args.payload;
    this.status = args.status;
    this.parentId = args.parentId;
    this.startUnixTimestamp = args.startUnixTimestamp;
    this.startTimestamp = args.startTimestamp;
    this.endUnixTimestamp = args.endUnixTimestamp;
    this.endTimestamp = args.endTimestamp;
    this.siStorable = args.siStorable;
    if (args.name) {
      this.name = args.name;
    } else {
      if (this.kind == EventKind.ResourceSync) {
        this.name = "Resource Sync";
      } else if (this.kind == EventKind.NodeEntityCreate) {
        this.name = "Node Created";
      } else {
        this.name = "Unknown";
      }
    }
  }

  localTime(): string {
    const dt = DateTime.fromMillis(this.startUnixTimestamp);
    const local = dt.toLocaleString(DateTime.DATETIME_SHORT_WITH_SECONDS);
    return local;
  }

  relativeToNow(): string {
    const dt = DateTime.fromMillis(this.startUnixTimestamp);
    const relative = dt.toRelative();
    if (relative) {
      return relative;
    } else {
      return this.startTimestamp;
    }
  }

  async save(): Promise<void> {
    const currentObj = await db.eventLog.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.events.put(this);
      this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    await store.dispatch("event/fromEvent", this, { root: true });
  }

  static async restore(): Promise<void> {
    let iObjects = await db.events.toArray();
    for (const iobj of iObjects) {
      let obj = new Event(iobj);
      await obj.dispatch();
    }
  }

  static async listForContext(context: string[]): Promise<Event[]> {
    const items: IEvent[] = await db.events
      .where("context")
      .anyOf(context)
      .toArray();
    return items.map(obj => new Event(obj));
  }

  async loadOwner(): Promise<void> {
    if (this.siStorable.createdByUserId) {
      const user = await User.get({ id: this.siStorable.createdByUserId });
      this.owner = user.name;
    } else {
      this.owner = "undefined";
    }
  }
}

db.events.mapToClass(Event);
