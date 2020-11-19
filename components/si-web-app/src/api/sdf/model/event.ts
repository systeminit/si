import _ from "lodash";
import { DateTime } from "luxon";

import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import store from "@/store";
import { User } from "./user";
import {
  IGetReply,
  IGetRequest,
  IListRequest,
  IListReply,
} from "@/api/sdf/model";
import { sdf } from "@/api/sdf";
import { Query, BooleanTerm, Comparison, FieldType } from "./query";

export enum EventStatus {
  Unknown = "unknown",
  Running = "running",
  Success = "success",
  Error = "error",
}

export enum EventKind {
  ResourceSync = "resourceSync",
  NodeEntityCreate = "nodeEntityCreate",
  EntityAction = "entityAction",
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
      } else if (this.kind == EventKind.EntityAction) {
        this.name = `Action ${this.payload["action"]}`;
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
    const queryItems = [];
    for (const value of context) {
      queryItems.push({
        expression: {
          field: "context",
          value,
          comparison: Comparison.Contains,
          fieldType: FieldType.String,
        },
      });
    }
    const query = new Query({
      booleanTerm: BooleanTerm.Or,
      items: queryItems,
    });
    const listReply = await Event.list({ query, pageSize: 500 });
    return listReply.items;
    //const items: IEvent[] = await db.events
    //  .where("context")
    //  .anyOf(context)
    //  .toArray();
    //return items.map(obj => new Event(obj));
  }

  static async list(request?: IListRequest): Promise<IListReply<Event>> {
    const items: Event[] = [];
    let totalCount = 0;

    //db.events.each(obj => {
    //  items.push(new Event(obj));
    //  totalCount++;
    //});

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<IEvent> = await sdf.list("events", request);
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new Event(item);
            objItem.save();
            items.push(objItem);
          }
        }
        if (reply.pageToken) {
          request = {
            pageToken: reply.pageToken,
          };
        } else {
          totalCount = reply.totalCount;
          finished = true;
        }
      }
    }
    return {
      items,
      totalCount,
    };
  }

  static async get(request: IGetRequest<IEvent["id"]>): Promise<Event> {
    const event = await db.events.get(request.id);
    if (event) {
      return new Event(event);
    }
    const reply: IGetReply<IEvent> = await sdf.get(`events/${request.id}`);
    const fetched: Event = new Event(reply.item);
    await fetched.save();
    return fetched;
  }

  async loadOwner(): Promise<void> {
    if (this.siStorable.createdByUserId) {
      const user = await User.get({ id: this.siStorable.createdByUserId });
      this.owner = user.name;
    } else {
      this.owner = "undefined";
    }
  }

  async parents(): Promise<Event[]> {
    const parents = [];
    if (this.parentId) {
      const checkForParent = [this.parentId];
      for (const parentId of checkForParent) {
        const parent = await Event.get({ id: parentId });
        if (parent) {
          parents.push(parent);
          if (parent.parentId) {
            console.log("have a parent", { parent, checkForParent });
            if (!checkForParent.includes(parent.parentId)) {
              console.log("but I didn't make it here");
              checkForParent.push(parent.parentId);
            }
          }
        }
      }
    }
    return parents;
  }
}

db.events.mapToClass(Event);
