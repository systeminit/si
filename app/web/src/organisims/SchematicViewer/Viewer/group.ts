import * as PIXI from "pixi.js";

import _ from "lodash";

export class Group extends PIXI.Container {
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
