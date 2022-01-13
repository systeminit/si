import * as PIXI from "pixi.js";

import { SceneManager } from "../scene";
import { SchematicDataManager } from "../../data";
import * as OBJ from "../obj";

interface Position {
  x: number;
  y: number;
}

export interface ConnectingInteractionData {
  position: {
    mouse: {
      x: number;
      y: number;
    };
  };
}

export class ConnectingManager {
  dataManager: SchematicDataManager;
  zoomFactor: number;
  interactiveConnection?: undefined;
  sourceSocket?: string | undefined;
  destinationSocket?: string | undefined;
  connection?: OBJ.Connection | undefined | null;
  data?: PIXI.InteractionData | undefined;
  offset?: Position | undefined;

  constructor(dataManager: SchematicDataManager) {
    this.dataManager = dataManager;
    this.zoomFactor = 1;
  }

  beforeConnect(
    data: PIXI.InteractionData,
    target: OBJ.Connection,
    sceneManager: SceneManager,
    offset: Position,
    zoomFactor: number,
  ): void {
    this.zoomFactor = zoomFactor;
    this.data = data;
    this.sourceSocket = target.name;
    this.offset = {
      x: offset.x,
      y: offset.y,
    };

    //  (sp.x - offset.x) * (1 / this.zoomFactor),

    sceneManager.interactiveConnection = sceneManager.createConnection(
      {
        x: (target.worldTransform.tx - offset.x) * (1 / this.zoomFactor),
        y: (target.worldTransform.ty - offset.y) * (1 / this.zoomFactor),
      },
      {
        x: (data.global.x - offset.x) * (1 / this.zoomFactor),
        y: (data.global.y - offset.y) * (1 / this.zoomFactor),
      },
      "none",
      "none",
      true,
    );
  }

  drag(data: PIXI.InteractionData, sceneManager: SceneManager): void {
    if (sceneManager.interactiveConnection && this.offset) {
      sceneManager.updateConnectionInteractive(
        sceneManager.interactiveConnection.name,
        {
          x: (data.global.x - this.offset.x) * (1 / this.zoomFactor),
          y: (data.global.y - this.offset.y) * (1 / this.zoomFactor),
        },
      );
      sceneManager.refreshConnections();
    }
  }

  connect(socket: string): void {
    this.destinationSocket = socket;
  }

  afterConnect(sceneManager: SceneManager): void {
    if (this.sourceSocket && this.destinationSocket && this.offset) {
      const source = sceneManager.getGeo(this.sourceSocket);
      const destination = sceneManager.getGeo(this.destinationSocket);
      sceneManager.createConnection(
        {
          x: (source.worldTransform.tx - this.offset.x) * (1 / this.zoomFactor),
          y: (source.worldTransform.ty - this.offset.y) * (1 / this.zoomFactor),
        },
        {
          x:
            (destination.worldTransform.tx - this.offset.x) *
            (1 / this.zoomFactor),
          y:
            (destination.worldTransform.ty - this.offset.y) *
            (1 / this.zoomFactor),
        },
        source.name,
        destination.name,
      );
      this.clearInteractiveConnection(sceneManager);
      sceneManager.refreshConnections();

      const sourceSocket = source.name.split(".");
      const destinationSocket = destination.name.split(".");

      this.dataManager.connectionCreate$.next({
        sourceNodeId: parseInt(sourceSocket[0]),
        sourceSocketId: parseInt(sourceSocket[1]),
        destinationNodeId: parseInt(destinationSocket[0]),
        destinationSocketId: parseInt(destinationSocket[1]),
      });
    }
  }

  clearInteractiveConnection(sceneManager: SceneManager): void {
    if (sceneManager.interactiveConnection) {
      sceneManager.removeConnection(sceneManager.interactiveConnection.name);
    }
  }
}
