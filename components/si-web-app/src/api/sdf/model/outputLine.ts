import _ from "lodash";

import { db } from "@/api/sdf/dexie";
import {
  IGetReply,
  IGetRequest,
  IListRequest,
  IListReply,
} from "@/api/sdf/model";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import store from "@/store";
import { sdf } from "@/api/sdf";
import { Comparison, FieldType } from "./query";

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

    if (items.length) {
      return items.map(obj => new OutputLine(obj));
    } else {
      const result = await OutputLine.list({
        query: {
          items: [
            {
              expression: {
                field: "eventLogId",
                value: eventLogId,
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
          ],
        },
        pageSize: 500,
      });
      return result.items;
    }
  }

  static async list(request?: IListRequest): Promise<IListReply<OutputLine>> {
    const items: OutputLine[] = [];
    let totalCount = 0;
    let finished = false;
    while (!finished) {
      const reply: IListReply<IOutputLine> = await sdf.list(
        "outputLines",
        request,
      );
      if (reply.items.length) {
        for (let item of reply.items) {
          let objItem = new OutputLine(item);
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
    return {
      items,
      totalCount,
    };
  }

  static async get(
    request: IGetRequest<IOutputLine["id"]>,
  ): Promise<OutputLine> {
    const event = await db.outputLines.get(request.id);
    if (event) {
      return new OutputLine(event);
    }
    const reply: IGetReply<IOutputLine> = await sdf.get(
      `outputLines/${request.id}`,
    );
    const fetched: OutputLine = new OutputLine(reply.item);
    await fetched.save();
    return fetched;
  }
}

db.outputLines.mapToClass(OutputLine);
