import _ from "lodash";
import { DateTime } from "luxon";

import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import store from "@/store";
import { sdf } from "@/api/sdf";

export enum EventLogLevel {
  Trace = "trace",
  Debug = "debug",
  Info = "info",
  Warn = "warn",
  Error = "error",
  Fatal = "fatal",
}

export interface IEventLog {
  id: string;
  message: string;
  unixTimestamp: number;
  timestamp: string;
  siStorable: ISiStorable;
  payload: any;
  level: EventLogLevel;
  eventId: string;
}

export class EventLog implements IEventLog {
  id: IEventLog["id"];
  message: IEventLog["message"];
  unixTimestamp: IEventLog["unixTimestamp"];
  timestamp: IEventLog["timestamp"];
  siStorable: IEventLog["siStorable"];
  payload: IEventLog["payload"];
  level: IEventLog["level"];
  eventId: IEventLog["eventId"];

  constructor(args: IEventLog) {
    this.id = args.id;
    this.message = args.message;
    this.unixTimestamp = args.unixTimestamp;
    this.timestamp = args.timestamp;
    this.siStorable = args.siStorable;
    this.payload = args.payload;
    this.level = args.level;
    this.eventId = args.eventId;
  }

  localTime(): string {
    const dt = DateTime.fromMillis(this.unixTimestamp);
    const local = dt.toLocaleString(DateTime.DATETIME_SHORT_WITH_SECONDS);
    return local;
  }

  relativeToNow(): string {
    const dt = DateTime.fromMillis(this.unixTimestamp);
    //const dt = DateTime.fromRFC2822(this.timestamp);
    const relative = dt.toRelative();
    if (relative) {
      return relative;
    } else {
      console.log("fuck you");
      return this.timestamp;
    }
  }

  async save(): Promise<void> {
    const currentObj = await db.eventLog.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.eventLog.put(this);
      this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    await store.dispatch("editor/fromEventLog", this, { root: true });
    await store.dispatch("event/fromEventLog", this, { root: true });
  }

  static async listForEvent(eventId: string): Promise<EventLog[]> {
    const items: IEventLog[] = await db.eventLog
      .where("eventId")
      .equals(eventId)
      .toArray();
    return items.map(obj => new EventLog(obj));
  }

  static async restore(): Promise<void> {
    let iObjects = await db.eventLog.toArray();
    for (const iobj of iObjects) {
      let obj = new EventLog(iobj);
      await obj.dispatch();
    }
  }
}
