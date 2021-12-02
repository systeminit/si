import * as PIXI from "pixi.js";

import { Node } from "../obj";
import { SceneManager } from "../scene";
import { SchematicDataManager } from "../../data";
import { NodeUpdate } from "../../model";

interface Position {
  x: number;
  y: number;
}

export interface DraggingInteractionData {
  position: {
    mouse: {
      x: number;
      y: number;
    };
  };
}

export class DraggingManager {
  sceneManager: SceneManager;
  dataManager: SchematicDataManager;
  data?: PIXI.InteractionData | undefined;
  offset?: Position | undefined;

  constructor(sceneManager: SceneManager, dataManager: SchematicDataManager) {
    this.sceneManager = sceneManager;
    this.dataManager = dataManager;
  }

  beforeDrag(data: PIXI.InteractionData, offset: Position): void {
    this.data = data;
    this.offset = offset;
  }

  drag(node: Node): void {
    if (this.data && this.offset) {
      const localPosition = this.data.getLocalPosition(node.parent);
      const position = {
        x: localPosition.x - this.offset.x,
        y: localPosition.y - this.offset.y,
      };

      this.sceneManager.translateNode(node, position);
    }
  }

  afterDrag(node: Node): void {
    const nodeUpdate: NodeUpdate = {
      nodeId: node.id,
      position: {
        ctxId: "aaa",
        x: node.x,
        y: node.y,
      },
    };
    this.dataManager.nodeUpdate$.next(nodeUpdate);
  }
}
