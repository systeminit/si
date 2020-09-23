import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import { ISiChangeSet } from "@/api/sdf/model/siChangeSet";
import _ from "lodash";

export interface ISystem {
  id: string;
  name: string;
  description: string;
  nodeId: string;
  head: boolean;
  siStorable: ISiStorable;
  siChangeSet: ISiChangeSet;
}

export class System implements ISystem {
  id: ISystem["id"];
  name: ISystem["name"];
  description: ISystem["description"];
  nodeId: ISystem["nodeId"];
  head: ISystem["head"];
  siStorable: ISystem["siStorable"];
  siChangeSet: ISystem["siChangeSet"];

  constructor(args: ISystem) {
    this.id = args.id;
    this.name = args.name;
    this.description = args.description;
    this.nodeId = args.nodeId;
    this.head = args.head;
    this.siStorable = args.siStorable;
    this.siChangeSet = args.siChangeSet;
  }

  async save(): Promise<void> {
    const currentObj = await db.systems.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.systems.put(this);
    }
  }
}

db.systems.mapToClass(System);
