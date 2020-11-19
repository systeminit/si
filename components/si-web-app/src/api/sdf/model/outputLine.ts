import _ from "lodash";

import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import store from "@/store";
import { sdf } from "@/api/sdf";

export enum OutputLineStream {
  Stdout = "stdout",
  Stderr = "stderr",
  All = "all",
}

export interface IOutputLine {
  id: string;
  line: string;
  stream: OutputLineStream;
  unixTimestamp: number;
  timestamp: string;
  siStorable: ISiStorable;
  eventId: string;
  eventLogId: string;
}

export class OutputLine implements IOutputLine {
  id: IOutputLine["id"];
  line: IOutputLine["line"];
  stream: IOutputLine["stream"];
  unixTimestamp: IOutputLine["unixTimestamp"];
  timestamp: IOutputLine["timestamp"];
  siStorable: IOutputLine["siStorable"];
  eventId: IOutputLine["eventId"];
  eventLogId: IOutputLine["eventLogId"];

  constructor(args: IOutputLine) {
    this.id = args.id;
    this.line = args.line;
    this.stream = args.stream;
    this.unixTimestamp = args.unixTimestamp;
    this.timestamp = args.timestamp;
    this.eventLogId = args.eventLogId;
    this.eventId = args.eventId;
    this.siStorable = args.siStorable;
  }

  async save(): Promise<void> {
    const currentObj = await db.outputLines.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.outputLines.put(this);
      this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    await store.dispatch("event/fromOutputLine", this, { root: true });
  }

  static async restore(): Promise<void> {
    let iObjects = await db.outputLines.toArray();
    for (const iobj of iObjects) {
      let obj = new OutputLine(iobj);
      await obj.dispatch();
    }
  }

  static async listForEventLog(eventLogId: string): Promise<OutputLine[]> {
    const items: IOutputLine[] = await db.outputLines
      .where("eventLogId")
      .equals(eventLogId)
      .toArray();
    return items.map(obj => new OutputLine(obj));
  }
}

db.outputLines.mapToClass(OutputLine);
