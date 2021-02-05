import { ISiStorable } from "@/api/sdf/model/siStorable";
import { ISiChangeSet } from "@/api/sdf/model/siChangeSet";
import _ from "lodash";
import Bottle from "bottlejs";

export interface IEntityGetReply {
  items: IEntity[];
}

export interface IEntityGetProjectionRequest {
  id: string;
  changeSetId: string;
}

export interface IEntity {
  id: string;
  name: string;
  description: string;
  objectType: string;
  expressionProperties: {
    [key: string]: Record<string, any>;
  };
  manualProperties: {
    [key: string]: Record<string, any>;
  };
  inferredProperties: {
    [key: string]: Record<string, any>;
  };
  properties: {
    [key: string]: Record<string, any>;
  };
  nodeId: string;
  head: boolean;
  base: boolean;
  siStorable: ISiStorable;
  siChangeSet: ISiChangeSet;
}

export class Entity implements IEntity {
  id: IEntity["id"];
  name: IEntity["name"];
  objectType: IEntity["objectType"];
  description: IEntity["description"];
  expressionProperties: IEntity["expressionProperties"];
  manualProperties: IEntity["manualProperties"];
  inferredProperties: IEntity["inferredProperties"];
  properties: IEntity["properties"];
  nodeId: IEntity["nodeId"];
  head: IEntity["head"];
  base: IEntity["base"];
  siStorable: IEntity["siStorable"];
  siChangeSet: IEntity["siChangeSet"];

  constructor(args: IEntity) {
    this.id = args.id;
    this.name = args.name;
    this.objectType = args.objectType;
    this.description = args.description;
    this.expressionProperties = args.expressionProperties;
    this.manualProperties = args.manualProperties;
    this.inferredProperties = args.inferredProperties;
    this.properties = args.properties;
    this.nodeId = args.nodeId;
    this.head = args.head;
    this.base = args.base;
    this.siStorable = args.siStorable;
    this.siChangeSet = args.siChangeSet;
  }

  static upgrade(obj: Entity | IEntity): Entity {
    if (obj instanceof Entity) {
      return obj;
    } else {
      return new Entity(obj);
    }
  }

  async updateStores() {
    const bottle = Bottle.pop("default");
    const store = bottle.container.Store;

    if (this.objectType == "application") {
      await store.dispatch("application/fromApplicationEntity", this);
    }
  }
}
