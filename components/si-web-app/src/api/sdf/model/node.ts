import { IEntity, Entity } from "@/api/sdf/model/entity";
import { ISystem, System } from "@/api/sdf/model/system";
import { ISiStorable } from "@/api/sdf/model/siStorable";

import _ from "lodash";

export type INodeObject = IEntity | ISystem;
export type NodeObject = Entity | System;

export enum NodeKind {
  Entity = "entity",
  System = "system",
}

export interface NodePosition {
  [key: string]: {
    x: string;
    y: string;
  };
}

export interface INode {
  id: string;
  positions: NodePosition;
  kind: NodeKind;
  objectType: string;
  siStorable: ISiStorable;
}

export class Node implements INode {
  id: INode["id"];
  positions: INode["positions"];
  kind: INode["kind"];
  objectType: INode["objectType"];
  siStorable: INode["siStorable"];

  constructor(args: INode) {
    this.id = args.id;
    this.positions = args.positions;
    this.kind = args.kind;
    this.objectType = args.objectType;
    this.siStorable = args.siStorable;
  }
}
