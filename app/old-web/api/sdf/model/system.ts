import { ISiStorable } from "@/api/sdf/model/siStorable";
import { ISiChangeSet } from "@/api/sdf/model/siChangeSet";
// import {
//   IListRequest,
//   IListReply,
//   IGetRequest,
//   IGetReply,
// } from "@/api/sdf/model";
import _ from "lodash";
// import Bottle from "bottlejs";

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

  static upgrade(obj: System | ISystem): System {
    if (obj instanceof System) {
      return obj;
    } else {
      return new System(obj);
    }
  }
}
