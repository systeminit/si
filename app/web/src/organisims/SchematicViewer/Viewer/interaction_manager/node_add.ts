import * as PIXI from "pixi.js";
import * as Rx from "rxjs";
import { Node } from "../obj/node";

import { SceneManager } from "../scene_manager";
import { SchematicDataManager } from "../../data_manager";
import { selectNode } from "@/observable/selection";

interface NodeAddState {
  node: Node;
  // Note: this probably needs to not be data on this object, and instead be part of the
  // node template/node somewhere. :)
  nodeAddSchemaId: number;
}

export class NodeAddManager {
  sceneManager: SceneManager;
  dataManager: SchematicDataManager;
  data?: PIXI.InteractionData;
  state?: NodeAddState;

  constructor(sceneManager: SceneManager, dataManager: SchematicDataManager) {
    this.sceneManager = sceneManager;
    this.dataManager = dataManager;
    this.data = undefined;
    this.state = undefined;
  }

  beforeAddNode(data: PIXI.InteractionData): void {
    this.data = data;
  }

  async addNode(nodeObj: Node, schemaId: number): Promise<void> {
    const schematicKind = await Rx.firstValueFrom(
      this.dataManager.schematicKind$,
    );

    this.sceneManager.addNode(nodeObj);
    this.state = {
      nodeAddSchemaId: schemaId,
      node: this.sceneManager.getGeo(nodeObj.name) as Node,
    };

    if (schematicKind) {
      const parentDeploymentNodeId = await Rx.firstValueFrom(
        this.dataManager.selectedDeploymentNodeId$,
      );
      await selectNode(this.state.node.id, parentDeploymentNodeId);
    }
  }

  drag(): void {
    if (this.data && this.state) {
      const positionOffset = {
        x: this.state.node.width * 0.5,
        y: this.state.node.height * 0.5,
      };

      const localPosition = this.data.getLocalPosition(this.state.node.parent);
      const position = {
        x: localPosition.x - positionOffset.x,
        y: localPosition.y - positionOffset.y,
      };
      this.sceneManager.translateNode(this.state.node, position);
      this.sceneManager.renderer.renderStage();
    }
  }

  async afterAddNode() {
    const editorContext = await Rx.firstValueFrom(
      this.dataManager.editorContext$,
    );
    const parentDeploymentNodeId = await Rx.firstValueFrom(
      this.dataManager.selectedDeploymentNodeId$,
    );
    if (this.state && editorContext) {
      this.dataManager.createNode({
        nodeSchemaId: this.state.nodeAddSchemaId,
        systemId: editorContext.systemId,
        x: `${this.state.node.position.x}`,
        y: `${this.state.node.position.y}`,
        parentNodeId: parentDeploymentNodeId,
      });
    }
    this.state = undefined;
  }
}
