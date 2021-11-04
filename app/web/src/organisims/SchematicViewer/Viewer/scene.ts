import * as PIXI from "pixi.js";
import { Node, NodeData, Connection, ConnectionType } from "./geo";
import { Group } from "./group";
// import { onMouseDown, onMouseMove } from "./interaction";
import { Renderer } from "./renderer";
import { zoomMagnitude$ } from "../state";
import { Grid, BACKGROUND_GRID_NAME } from "./geo";
// import { Interpreter } from "xstate";

// interface Position {
//   x: number;
//   y: number;
// }

interface Point {
  x: number;
  y: number;
}

export interface SceneGraphData {
  nodes: Array<NodeData>;
}

export interface SceneData {
  nodes: Array<NodeData>;
}

export class SceneManager {
  renderer: Renderer;
  scene: PIXI.Container;
  root: PIXI.Container;
  interactiveConnection: Connection | null | undefined;
  group?: {
    nodes: PIXI.Container;
    connections: PIXI.Container;
  };
  zoomMagnitude: number | null;

  constructor(renderer: Renderer, data: SceneGraphData) {
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

    this.zoomMagnitude = 0;
    zoomMagnitude$.subscribe({ next: (v) => (this.zoomMagnitude = v) });

    this.loadSceneData(data);
  }

  updateBackgroundGrid(_zoomMagnitude: number) {
    // Not implemented yet
    const _grid = this.root.getChildByName(BACKGROUND_GRID_NAME, true);
    // grid.updateTransform(zoomMagnitude);
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
    this.group = {
      nodes: new Group("nodes", 20),
      connections: new Group("connections", 30),
    };
    if (this.group) {
      this.root.addChild(this.group.nodes);
      this.root.addChild(this.group.connections);
    }
  }

  loadSceneData(data: SceneGraphData): void {
    for (const n of data.nodes) {
      const node = new Node(n.name, n.position);
      this.addNode(node);
    }
  }

  reloadSceneData(data: SceneGraphData): void {
    this.clearSceneData();
    this.loadSceneData(data);
  }

  clearSceneData(): void {
    this.initializeSceneData();
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
    if (this.group && this.group.nodes) {
      this.group.nodes.addChild(n);
    }
  }

  addGrid(g: Grid): void {
    if (this.group && this.group.grid) {
      this.group.grid.addChild(g);
    }
  }

  removeNode(): void {
    console.log("removeNode");
  }

  createConnection(
    p1: Point,
    p2: Point,
    sourceSocketId: string,
    destinationSocketId: string,
    _interactive?: boolean,
  ): Connection | null {
    const connection = new Connection(
      p1,
      p2,
      sourceSocketId,
      destinationSocketId,
      _interactive,
    );

    let isConnectionUnique = true;
    if (this.group?.connections) {
      for (const c of this.group.connections.children) {
        const conn = c as Connection;
        if (conn.name === connection.name) {
          isConnectionUnique = false;
        }
      }
    }

    if (isConnectionUnique) {
      this.addConnection(connection);
      return connection;
    } else {
      console.log("connection already exist!");
      return null;
    }
  }
  addConnection(c: Connection): void {
    this.group?.connections.addChild(c);
  }

  removeConnection(name: string): void {
    const c = this.scene.getChildByName(name, true) as Connection;
    this.group?.connections.removeChild(c);
  }

  refreshConnections(): void {
    if (this.group?.connections) {
      for (const c of this.group.connections.children) {
        const connection = c as Connection;
        if (connection.type != ConnectionType.interactive)
          this.refreshConnectionPosition(c.name);
      }
    }
  }

  refreshConnectionPosition(name: string): void {
    const c = this.scene.getChildByName(name, true) as Connection;

    const sp = this.getSocketPosition(c.sourceSocketId);
    const dp = this.getSocketPosition(c.destinationSocketId);

    if (this.zoomMagnitude != null) {
      const offset = {
        x: this.root.x,
        y: this.root.y,
      };

      const p1 = {
        x: sp.x - offset.x,
        y: sp.y - offset.y,
      };

      const p2 = {
        x: dp.x - offset.x,
        y: dp.y - offset.y,
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
