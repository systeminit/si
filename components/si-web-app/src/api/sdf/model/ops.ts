import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import { ISiChangeSet } from "@/api/sdf/model/siChangeSet";
import store from "@/store";
import _ from "lodash";

export type IEntityOps = IOpEntitySet;

export interface IObjectPatchRequest {
  op: IOpRequest;
  organizationId: string;
  workspaceId: string;
  changeSetId: string;
  editSessionId: string;
}

export interface IOpRequest {
  entitySet?: {
    path: string[];
    value: any;
    overrideSystem?: string;
  };
  nameSet?: {
    value: string;
  };
  entityDelete?: {
    cascade: boolean;
  };
  entityAction?: {
    action: string;
    systemId: string;
  };
}

export interface IOpReply {
  itemIds: string[];
}

export interface ISiOp {
  skip: boolean;
  overrideSystem?: string;
}

export interface IOpEntitySet {
  id: string;
  toId: string;
  path: string[];
  value: any;
  siOp: ISiOp;
  siStorable: ISiStorable;
  siChangeSet: ISiChangeSet;
}

export class OpEntitySet implements IOpEntitySet {
  id: IOpEntitySet["id"];
  toId: IOpEntitySet["toId"];
  path: IOpEntitySet["path"];
  value: IOpEntitySet["value"];
  siOp: IOpEntitySet["siOp"];
  siStorable: IOpEntitySet["siStorable"];
  siChangeSet: IOpEntitySet["siChangeSet"];

  constructor(args: IOpEntitySet) {
    this.id = args.id;
    this.toId = args.toId;
    this.path = args.path;
    this.value = args.value;
    this.siOp = args.siOp;
    this.siStorable = args.siStorable;
    this.siChangeSet = args.siChangeSet;
  }

  static async create(
    nodeId: string,
    request: IObjectPatchRequest,
  ): Promise<void> {
    await sdf.patch(`nodes/${nodeId}/object`, request);
  }

  async save(): Promise<void> {
    const currentObj = await db.entityOps.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.entityOps.put(this);
    }
  }
}

db.entityOps.mapToClass(OpEntitySet);
