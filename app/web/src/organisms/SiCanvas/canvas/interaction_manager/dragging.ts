import * as PIXI from "pixi.js";

import { Node } from "../obj/node";
import { SceneManager } from "../scene_manager";
import { SchematicDataManager } from "../../data_manager";

interface DraggingState {
  data: PIXI.InteractionData;
  offset: Position;
}

export class DraggingManager {
  sceneManager: SceneManager;
  dataManager: SchematicDataManager;
  state?: DraggingState;

  constructor(sceneManager: SceneManager, dataManager: SchematicDataManager) {
    this.sceneManager = sceneManager;
    this.dataManager = dataManager;
    this.state = undefined;
  }

  beforeDrag(data: PIXI.InteractionData, offset: Position): void {
    this.state = {
      data,
      offset,
    };
  }

  drag(node: Node): void {
    if (this.state) {
      const localPosition = this.state.data.getLocalPosition(node.parent);
      const position = {
        x: localPosition.x - this.state.offset.x,
        y: localPosition.y - this.state.offset.y,
      };

      this.sceneManager.translateNode(node, position);
    }
  }

  afterDrag(node: Node): void {
    this.dataManager.updateNodePosition(node.id, node.x, node.y);
    this.state = undefined;
  }
}

interface Position {
  x: number;
  y: number;
}
