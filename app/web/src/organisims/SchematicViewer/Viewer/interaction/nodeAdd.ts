import * as PIXI from "pixi.js";

import { SceneManager } from "../scene";
import { SchematicDataManager } from "../../data";
import { Renderer } from "../renderer";
import * as OBJ from "../obj";
import * as MODEL from "../../model";
import { NodeCreate } from "../../data/event";
import {EditorContext} from "@/api/sdf/dal/schematic";
import { refFrom } from "vuse-rx";

import { selection$ } from "../../state";

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
  renderer: Renderer;
  data?: PIXI.InteractionData | undefined;
  node?: OBJ.Node;
  // Note: this probably needs to not be data on this object, and instead be part of the
  // node template/node somewhere. :)
  nodeAddSchemaId?: number;
  editorContext: refFrom<EditorContext | null>;

  constructor(
    sceneManager: SceneManager,
    dataManager: SchematicDataManager,
    renderer: Renderer,
    editorContext: refFrom<EditorContext | null>,
  ) {
    this.sceneManager = sceneManager;
    this.dataManager = dataManager;
    this.renderer = renderer;
    this.editorContext = editorContext;
  }

  beforeAddNode(data: PIXI.InteractionData): void {
    this.data = data;
  }

  addNode(n: MODEL.Node, schemaId: number): void {
    const nodeObj = new OBJ.Node(n);
    this.sceneManager.addNode(nodeObj);
    this.nodeAddSchemaId = schemaId;
    this.node = this.sceneManager.getGeo(nodeObj.name) as OBJ.Node;
    this.select(this.node);
  }

  select(node: OBJ.Node): void {
    node.zIndex += 1;
    const selection = [];
    selection.push(node);
    selection$.next(selection);
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

  afterAddNode(): void {
    if (this.node && this.nodeAddSchemaId && this.editorContext) {
      const event: NodeCreate = {
        nodeSchemaId: this.nodeAddSchemaId,
        rootNodeId: this.editorContext.applicationNodeId,
        systemId: this.editorContext.systemId,
        x: `${this.node.position.x}`,
        y: `${this.node.position.y}`,
      };

      this.dataManager.nodeCreate$.next(event);

      // TODO waiting for backend to implement "node swap". A schematic reload shuld be fine.
      this.sceneManager.removeNode(this.node);
      this.sceneManager.renderer.renderStage();

      // cleanup
      this.node = undefined;
      this.nodeAddSchemaId = undefined;
      selection$.next(null);
    }
  }
}
