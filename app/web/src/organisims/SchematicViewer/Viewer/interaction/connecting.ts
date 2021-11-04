import * as PIXI from "pixi.js";

import { SceneManager } from "../scene";
import { Connection } from "../geo";

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
  interactiveConnection?: undefined;
  sourceSocket?: string | undefined;
  destinationSocket?: string | undefined;
  connection?: Connection | undefined | null;
  data?: PIXI.InteractionData | undefined;
  offset?: Position | undefined;

  constructor() {}

  beforeConnect(
    data: PIXI.InteractionData,
    target: PIXI.Container,
    sceneManager: SceneManager,
    offset: Position,
  ): void {
    this.data = data;
    this.sourceSocket = target.name;
    this.offset = {
      x: offset.x,
      y: offset.y,
    };

    sceneManager.interactiveConnection = sceneManager.createConnection(
      {
        x: target.worldTransform.tx - offset.x,
        y: target.worldTransform.ty - offset.y,
      },
      {
        x: data.global.x - offset.x,
        y: data.global.y - offset.y,
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
          x: data.global.x - this.offset.x,
          y: data.global.y - this.offset.y,
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
          x: source.worldTransform.tx - this.offset.x,
          y: source.worldTransform.ty - this.offset.y,
        },
        {
          x: destination.worldTransform.tx - this.offset.x,
          y: destination.worldTransform.ty - this.offset.y,
        },
        source.name,
        destination.name,
      );
      this.clearInteractiveConnection(sceneManager);
      sceneManager.refreshConnections();
    }
  }

  clearInteractiveConnection(sceneManager: SceneManager): void {
    if (sceneManager.interactiveConnection) {
      sceneManager.removeConnection(sceneManager.interactiveConnection.name);
    }
  }
}
