import * as PIXI from "pixi.js";

import { Sockets } from "./obj/node/sockets";
import { Connection, ConnectionType } from "./obj/connection";
import { Socket } from "./obj/node/sockets/socket";
import { Node } from "./obj/node";
import { Grid, BACKGROUND_GRID_NAME } from "./obj/grid";
import { SchematicGroup, NodeGroup, ConnectionGroup } from "./obj/group";
import { Renderer } from "./renderer";
import { untilUnmounted } from "vuse-rx";
import { InteractionManager } from "./interaction_manager";
import { SchematicKind } from "@/api/sdf/dal/schematic";
import {
  Schematic,
  variantById,
  inputSocketById,
} from "@/api/sdf/dal/schematic";

export type SceneGraphData = Schematic;

interface Point {
  x: number;
  y: number;
}

export class SceneManager {
  renderer: Renderer;
  scene: PIXI.Container;
  root: PIXI.Container;
  interactiveConnection?: Connection | null;
  group: {
    nodes: PIXI.Container;
    connections: PIXI.Container;
  };
  zoomFactor?: number;

  constructor(renderer: Renderer, lightMode: boolean) {
    this.renderer = renderer;
    this.scene = new PIXI.Container();
    this.scene.name = "scene";
    this.scene.interactive = true;
    this.scene.sortableChildren = true;

    this.scene.hitArea = new PIXI.Rectangle(
      0,
      0,
      renderer.width,
      renderer.height,
    );

    this.root = new PIXI.Container();
    this.root.name = "root";
    this.root.sortableChildren = true;
    this.root.zIndex = 2;
    this.scene.addChild(this.root);

    this.group = {
      connections: new ConnectionGroup("connections", 0),
      nodes: new NodeGroup("nodes", 0),
    };

    this.initializeSceneData();
    this.setBackgroundGrid(renderer.width, renderer.height, lightMode);

    this.zoomFactor = 1;
  }

  subscribeToInteractionEvents(interactionManager: InteractionManager) {
    interactionManager.zoomFactor$
      .pipe(untilUnmounted)
      .subscribe((v) => this.updateZoomFactor(v));
  }

  updateZoomFactor(zoomFactor: number | null) {
    if (zoomFactor) {
      this.zoomFactor = zoomFactor;
      const grid = this.root.getChildByName(BACKGROUND_GRID_NAME, true) as Grid;
      grid.updateZoomFactor(zoomFactor);
      grid.render(this.renderer);
    }
  }

  setBackgroundGrid(
    rendererWidth: number,
    rendererHeight: number,
    lightMode: boolean,
  ): void {
    const grid = new Grid(rendererWidth, rendererHeight, lightMode);
    grid.zIndex = 1;
    this.root.addChild(grid);
  }

  initializeSceneData(): void {
    this.clearSceneData();

    this.group = {
      connections: new ConnectionGroup("connections", 20),
      nodes: new NodeGroup("nodes", 30),
    };
    this.root.addChild(this.group.nodes);
    this.root.addChild(this.group.connections);
  }

  async loadSceneData(
    data: Schematic | null,
    schematicKind: SchematicKind,
    selectedDeploymentNodeId: number | null,
  ): Promise<void> {
    this.initializeSceneData();

    if (data) {
      for (const n of data.nodes) {
        const variant = await variantById(n.schemaVariantId);

        const pos = n.positions.find(
          (pos) =>
            pos.schematicKind === schematicKind &&
            pos.deploymentNodeId === selectedDeploymentNodeId,
        );
        if (pos) {
          const node = new Node(
            n,
            variant,
            {
              x: pos.x,
              y: pos.y,
            },
            schematicKind,
          );
          this.addNode(node);
        } else {
          // console.error("Node didn't have a position:", n);
        }
      }

      for (const connection of data.connections) {
        const sourceSocketId = `${connection.sourceNodeId}.${connection.sourceSocketId}`;
        const sourceSocket = this.scene.getChildByName(
          sourceSocketId,
          true,
        ) as Socket;

        const destinationSocketId = `${connection.destinationNodeId}.${connection.destinationSocketId}`;
        const destinationSocket = this.scene.getChildByName(
          destinationSocketId,
          true,
        );

        // Sometimes the connection isn't valid for display, like when switching panels while rendering
        // And the "include" connections also won't be found as they don't get rendered, we could use some metadata,
        // but there isn't much to gain from it
        if (!sourceSocket || !destinationSocket) continue;

        const socket = await inputSocketById(sourceSocket.id);
        this.createConnection(
          sourceSocket.getGlobalPosition(),
          destinationSocket.getGlobalPosition(),
          sourceSocket.name,
          destinationSocket.name,
          socket.provider.color,
        );
      }
    }

    this.renderer.renderStage();
  }

  clearSceneData(): void {
    for (let i = 0; i < this.root.children.length; i++) {
      const group = this.root.children[i] as SchematicGroup | Grid;
      if (group instanceof NodeGroup || group instanceof ConnectionGroup) {
        this.root.removeChild(group);
      }
    }
  }

  getSocketPosition(socketId: string): PIXI.Point {
    const socket = this.scene.getChildByName(socketId, true);
    const position = socket.getGlobalPosition();
    return position;
  }

  getGeo(name: string): PIXI.DisplayObject {
    const geo = this.renderer.stage.getChildByName(name, true);
    return geo;
  }

  addNode(n: Node): void {
    this.group.nodes.addChild(n);
  }

  translateNode(node: Node, position: Point): void {
    node.x = position.x;
    node.y = position.y;
    node.updateTransform();
  }

  createConnection(
    p1: Point,
    p2: Point,
    sourceSocketId: string,
    destinationSocketId: string,
    color: number,
    interactive?: boolean,
  ): Connection | null {
    const connection = new Connection(
      p1,
      p2,
      sourceSocketId,
      destinationSocketId,
      color,
      interactive,
    );
    let isConnectionUnique = true;
    for (const c of this.group.connections.children) {
      const conn = c as Connection;
      if (conn.name === connection.name) {
        isConnectionUnique = false;
      }
    }

    for (const n of this.group.nodes.children) {
      const node = n as Node;
      for (const sockets of node.children) {
        if (sockets instanceof Sockets) {
          const source = sockets.getChildByName(sourceSocketId) as Socket;
          if (source) source.setConnected();

          const destination = sockets.getChildByName(
            destinationSocketId,
          ) as Socket;
          if (destination) destination.setConnected();
        }
      }
    }

    if (isConnectionUnique) {
      this.addConnection(connection);
      this.refreshConnections(); // inefficient, should be for the connections on a node.
      return connection;
    } else {
      return null;
    }
  }

  addConnection(c: Connection): void {
    this.group.connections.addChild(c);
  }

  removeConnection(name: string): void {
    const c = this.scene.getChildByName(name, true) as Connection;
    this.group.connections.removeChild(c);
  }

  refreshConnections(): void {
    for (const c of this.group.connections.children) {
      const connection = c as Connection;
      if (connection && connection.type != ConnectionType.interactive) {
        this.refreshConnectionPosition(connection.name);
      }
    }
  }

  refreshConnectionPosition(name: string): void {
    const c = this.scene.getChildByName(name, true) as Connection;
    const sp = this.getSocketPosition(c.sourceSocketId);
    const dp = this.getSocketPosition(c.destinationSocketId);

    //  target.worldTransform.tx) * (1 / zoomFactor)
    if (this.zoomFactor != null) {
      const offset = {
        x: this.root.x,
        y: this.root.y,
      };

      const p1 = {
        x: (sp.x - offset.x) * (1 / this.zoomFactor),
        y: (sp.y - offset.y) * (1 / this.zoomFactor),
      };

      const p2 = {
        x: (dp.x - offset.x) * (1 / this.zoomFactor),
        y: (dp.y - offset.y) * (1 / this.zoomFactor),
      };
      c.update(p1, p2);
    }
  }

  updateConnectionInteractive(name: string, p: Point): void {
    const c = this.scene.getChildByName(name, true) as Connection;

    if (c && this.interactiveConnection) {
      const p1 = {
        x: this.interactiveConnection.x,
        y: this.interactiveConnection.y,
      };
      const p2 = {
        x: p.x,
        y: p.y,
      };
      c.update(p1, p2);
    }
  }
}
