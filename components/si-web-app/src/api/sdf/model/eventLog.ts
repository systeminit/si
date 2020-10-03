import _ from "lodash";

import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import store from "@/store";
import { sdf } from "@/api/sdf";

export interface IEventLog {
  id: string;
  message: string;
  unixTimestamp: number;
  timestamp: string;
  siStorable: ISiStorable;
  payload: any;
}

export class EventLog implements IEventLog {
  id: IEventLog["id"];
  message: IEventLog["message"];
  unixTimestamp: IEventLog["unixTimestamp"];
  timestamp: IEventLog["timestamp"];
  siStorable: IEventLog["siStorable"];
  payload: IEventLog["payload"];

  constructor(args: IEventLog) {
    this.id = args.id;
    this.message = args.message;
    this.unixTimestamp = args.unixTimestamp;
    this.timestamp = args.timestamp;
    this.siStorable = args.siStorable;
    this.payload = args.payload;
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
  }

  static async restore(): Promise<void> {
    let iObjects = await db.eventLog.toArray();
    for (const iobj of iObjects) {
      let obj = new EventLog(iobj);
      await obj.dispatch();
    }
  }
}
