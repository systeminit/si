import * as PIXI from "pixi.js";
import * as OBJ from "../obj";

import { SchematicGroup, NodeGroup, ConnectionGroup } from "../group";
import { Renderer } from "../renderer";
import { Grid, BACKGROUND_GRID_NAME } from "../obj";
import { Schematic } from "../../model";
import { Position } from "../cg";
import { untilUnmounted } from "vuse-rx";
import { InteractionManager } from "../interaction";
import { SelectionManager } from "../interaction/selection";
import { schematicKindFromNodeKind } from "@/api/sdf/dal/schematic";

export type SceneGraphData = Schematic;

interface Point {
  x: number;
  y: number;
}

export class SceneManager {
  renderer: Renderer;
  scene: PIXI.Container;
  root: PIXI.Container;
  interactiveConnection: OBJ.Connection | null | undefined;
  group?: {
    nodes: PIXI.Container;
    connections: PIXI.Container;
  };
  zoomFactor?: number;

  constructor(renderer: Renderer) {
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

    this.initializeSceneData();
    this.setBackgroundGrid();

    this.zoomFactor = 1;
  }

  subscribeToInteractionEvents(interactionManager: InteractionManager) {
    interactionManager.zoomFactor$.pipe(untilUnmounted).subscribe({
      next: (v) => this.updateZoomFactor(v),
    });
  }

  updateZoomFactor(zoomFactor: number | null) {
    if (zoomFactor) {
      this.zoomFactor = zoomFactor;
      const grid = this.root.getChildByName(BACKGROUND_GRID_NAME, true) as Grid;
      grid.updateZoomFactor(zoomFactor);
      grid.render(this.renderer);
    }
  }

  setBackgroundGrid(): void {
    const viewport = {
      width: 800,
      height: 800,
    };

    const size = Math.max(viewport.width, viewport.height);
    const grid = new Grid(size);
    grid.zIndex = 1;

    grid.position.x = -(size * 0.5);
    grid.position.y = -(size * 0.5);

    this.root.addChild(grid);
  }

  initializeSceneData(): void {
    this.clearSceneData();

    this.group = {
      nodes: new NodeGroup("nodes", 20),
      connections: new ConnectionGroup("connections", 30),
    };
    this.root.addChild(this.group.nodes);
    this.root.addChild(this.group.connections);
  }

  loadSceneData(
    data: SceneGraphData | null,
    selectionManager: SelectionManager,
  ): void {
    this.initializeSceneData();

    let selected;
    if (data) {
      for (const n of data.nodes) {
        if (n.position.length > 0) {
          const node = new OBJ.Node(n);
          // If the node was previously selected we re-select again as some operations
          // were lost on the re-render (example: update node position)
          if (node.id === (selectionManager.selection[0] ?? {}).id) {
            selected = node;
          }
          this.addNode(node);
        } else {
          // console.error("Node didn't have a position:", n);
        }
      }

      if (data.connections.length > 0) {
        for (const connection of data.connections) {
          if (connection.classification === "configures") {
            const sourceSocketId = `${connection.source.nodeId}.${connection.source.socketId}`;
            const sourceSocket = this.scene.getChildByName(
              sourceSocketId,
              true,
            );
            // Note: this happens when we switch panels with a connection rendered
            // The nodes and the connections don't disappear
            // We need to understand this better, but continuing here works by now, it may leak something tho
            if (!sourceSocket) continue;

            const destinationSocketId = `${connection.destination.nodeId}.${connection.destination.socketId}`;
            const destinationSocket = this.scene.getChildByName(
              destinationSocketId,
              true,
            );

            this.createConnection(
              sourceSocket.getGlobalPosition(),
              destinationSocket.getGlobalPosition(),
              sourceSocket.name,
              destinationSocket.name,
            );
          }
        }
      }
    }

    if (selected?.nodeKind) {
      const selectionObserver = selectionManager.selectionObserver(
        schematicKindFromNodeKind(selected.nodeKind.kind),
      );
      console.debug(
        "Re-selecting node: " +
          schematicKindFromNodeKind(selected.nodeKind.kind),
      );
      selectionManager.select(selected, selectionObserver);
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

  addNode(n: OBJ.Node): void {
    if (this.group && this.group.nodes) {
      this.group.nodes.addChild(n);
    }
  }

  removeNode(node: OBJ.Node): void {
    node.destroy();

    if (this.group) {
      const nodeGroup = this.scene.getChildByName(this.group.nodes.name, true);
      this.renderer.renderGroup(nodeGroup);
    }
  }

  translateNode(node: OBJ.Node, position: Position): void {
    node.x = position.x;
    node.y = position.y;
    node.updateTransform();
  }

  createConnection(
    p1: Point,
    p2: Point,
    sourceSocketId: string,
    destinationSocketId: string,
    _interactive?: boolean,
  ): OBJ.Connection | null {
    const connection = new OBJ.Connection(
      p1,
      p2,
      sourceSocketId,
      destinationSocketId,
      _interactive,
    );
    let isConnectionUnique = true;
    if (this.group?.connections) {
      for (const c of this.group.connections.children) {
        const conn = c as OBJ.Connection;
        if (conn.name === connection.name) {
          isConnectionUnique = false;
        }
      }
    }

    if (isConnectionUnique) {
      this.addConnection(connection);
      this.refreshConnections(); // inefficient, should be for the connections on a node.
      // this.renderConnection(connection); // causes an orphan edge to renders.
      return connection;
    } else {
      return null;
    }
  }

  addConnection(c: OBJ.Connection): void {
    this.group?.connections.addChild(c);
  }

  removeConnection(name: string): void {
    const c = this.scene.getChildByName(name, true) as OBJ.Connection;
    this.group?.connections.removeChild(c);
  }

  refreshConnections(): void {
    if (this.group?.connections) {
      for (const c of this.group.connections.children) {
        const connection = c as OBJ.Connection;
        if (connection && connection.type != OBJ.ConnectionType.interactive) {
          this.refreshConnectionPosition(connection.name);
        }
      }
    }
  }

  refreshConnectionPosition(name: string): void {
    const c = this.scene.getChildByName(name, true) as OBJ.Connection;
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
    const c = this.scene.getChildByName(name, true) as OBJ.Connection;

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

  getConnections(): void {
    const connections = this.group?.connections.children;
    console.log(connections);
  }

  renderConnection(c: OBJ.Connection): void {
    c.render(this.renderer);
  }
}
