import * as PIXI from "pixi.js";
import * as Rx from "rxjs";
import * as OBJ from "../obj";

import { SceneManager } from "../scene";
import { SchematicDataManager } from "../../data";
import { SelectionManager } from "./selection";
import { Renderer } from "../renderer";
import { NodeCreate } from "../../data/event";

export interface NodeAddInteractionData {
  position: {
    mouse: {
      x: number;
      y: number;
    };
  };
}

export class NodeAddManager {
  sceneManager: SceneManager;
  dataManager: SchematicDataManager;
  selectionManager: SelectionManager;
  renderer: Renderer;
  data?: PIXI.InteractionData | undefined;
  node?: OBJ.Node;
  // Note: this probably needs to not be data on this object, and instead be part of the
  // node template/node somewhere. :)
  nodeAddSchemaId?: number;

  constructor(
    sceneManager: SceneManager,
    dataManager: SchematicDataManager,
    selectionManager: SelectionManager,
    renderer: Renderer,
  ) {
    this.sceneManager = sceneManager;
    this.dataManager = dataManager;
    this.selectionManager = selectionManager;
    this.renderer = renderer;
  }

  beforeAddNode(data: PIXI.InteractionData): void {
    this.data = data;
  }

  addNode(nodeObj: OBJ.Node, schemaId: number): void {
    this.sceneManager.addNode(nodeObj);
    this.nodeAddSchemaId = schemaId;
    this.node = this.sceneManager.getGeo(nodeObj.name) as OBJ.Node;

    // Since node doesn't exist yet let's not sync the node add
    this.selectionManager.select(this.node);
  }

  drag(): void {
    if (this.data && this.node) {
      const positionOffset = {
        x: this.node.width * 0.5,
        y: this.node.height * 0.5,
      };

      const localPosition = this.data.getLocalPosition(this.node.parent);
      const position = {
        x: localPosition.x - positionOffset.x,
        y: localPosition.y - positionOffset.y,
      };
      this.sceneManager.translateNode(this.node, position);
      this.sceneManager.renderer.renderStage();
    }
  }

  async afterAddNode() {
    const editorContext = await Rx.firstValueFrom(
      this.dataManager.editorContext$,
    );
    if (this.node && this.nodeAddSchemaId && editorContext) {
      const event: NodeCreate = {
        nodeSchemaId: this.nodeAddSchemaId,
        rootNodeId: editorContext.applicationNodeId,
        systemId: editorContext.systemId,
        x: `${this.node.position.x}`,
        y: `${this.node.position.y}`,
      };

      this.dataManager.nodeCreate$.next(event);

      // TODO waiting for backend to implement "node swap". A schematic reload shuld be fine.
      // this.sceneManager.removeNode(this.node);
      // this.sceneManager.renderer.renderStage();

      // cleanup
      this.node = undefined;
      this.nodeAddSchemaId = undefined;
    }
  }
}
