import * as PIXI from "pixi.js";

import _ from "lodash";

export enum GroupKind {
  NODES = "nodes",
  CONNECTIONS = "connections",
}

export class SceneGroup extends PIXI.Container {
  kind = "group";
  id: string;

  constructor(name: string, zIndex: number) {
    super();

    this.id = _.uniqueId();
    this.name = name + ":" + this.id;
    this.zIndex = zIndex;
    this.sortableChildren = true;
  }
}

export class NodeGroup extends SceneGroup {
  groupKind: GroupKind.NODES;

  constructor(name: string, zIndex: number) {
    super(name, zIndex);
    this.groupKind = GroupKind.NODES;
  }
}

export class ConnectionGroup extends SceneGroup {
  groupKind: GroupKind.CONNECTIONS;

  constructor(name: string, zIndex: number) {
    super(name, zIndex);
    this.groupKind = GroupKind.CONNECTIONS;
  }
}

export type SchematicGroup = NodeGroup | ConnectionGroup;
