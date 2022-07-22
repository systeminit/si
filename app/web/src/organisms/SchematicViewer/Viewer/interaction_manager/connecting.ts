import * as PIXI from "pixi.js";
import { Node } from "../obj/node";
import _ from "lodash";

import { Socket } from "../obj/node/sockets/socket";
import { SceneManager } from "../scene_manager";
import { SchematicDataManager } from "../../data_manager";
import {
  variantById,
  inputSocketByVariantAndProvider,
  inputSocketById,
  outputSocketById,
} from "@/api/sdf/dal/schematic";

interface ConnectionState {
  sourceSocket: string;
  destinationSocket?: string;
  data: PIXI.InteractionData;
  offset: Position;
  target: Socket;
  targetWasConnected: boolean;
  createdConnection: boolean;
  zoomFactor: number;
}

export class ConnectingManager {
  dataManager: SchematicDataManager;
  state?: ConnectionState;

  constructor(dataManager: SchematicDataManager) {
    this.dataManager = dataManager;
    this.state = undefined;
  }

  async beforeConnect(
    data: PIXI.InteractionData,
    target: Socket,
    sceneManager: SceneManager,
    offset: Position,
    zoomFactor: number,
  ): Promise<void> {
    const rawSourceSocket = target.name;
    const sourceSocketId = parseInt(rawSourceSocket.split(".")[1]);
    const sourceSocket = await outputSocketById(sourceSocketId);
    const metadata = sourceSocket.provider.ty;

    this.state = {
      zoomFactor,
      data,
      sourceSocket: rawSourceSocket,
      offset, //  (sp.x - offset.x) * (1 / zoomFactor),
      target,
      targetWasConnected: target.isConnected(),
      createdConnection: false,
    };
    target.setConnected();

    const nodes = sceneManager.group.nodes.children as Node[] | undefined;
    for (const node of nodes ?? []) {
      const variant = await variantById(node.schemaVariantId);
      try {
        inputSocketByVariantAndProvider(variant, metadata);
      } catch {
        node.dim();
      }
    }

    sceneManager.interactiveConnection = sceneManager.createConnection(
      {
        x: (target.worldTransform.tx - offset.x) * (1 / zoomFactor),
        y: (target.worldTransform.ty - offset.y) * (1 / zoomFactor),
      },
      {
        x: (data.global.x - offset.x) * (1 / zoomFactor),
        y: (data.global.y - offset.y) * (1 / zoomFactor),
      },
      sourceSocket.name,
      "none",
      sourceSocket.provider.color,
      true,
    );
  }

  drag(data: PIXI.InteractionData, sceneManager: SceneManager): void {
    if (sceneManager.interactiveConnection && this.state) {
      sceneManager.updateConnectionInteractive(
        sceneManager.interactiveConnection.name,
        {
          x:
            (data.global.x - this.state.offset.x) * (1 / this.state.zoomFactor),
          y:
            (data.global.y - this.state.offset.y) * (1 / this.state.zoomFactor),
        },
      );
      sceneManager.refreshConnections();
    }
  }

  connect(socket: string): void {
    if (this.state) {
      this.state.destinationSocket = socket;
    }
  }

  async afterConnect(sceneManager: SceneManager): Promise<void> {
    if (this.state && this.state.destinationSocket) {
      const source = sceneManager.getGeo(this.state.sourceSocket);
      const destination = sceneManager.getGeo(this.state.destinationSocket);
      if (!source || !destination) return;

      const sourceSocketStr = source.name.split(".");
      const sourceNodeId = parseInt(sourceSocketStr[0]);
      const sourceSocketId = parseInt(sourceSocketStr[1]);

      const destinationSocketStr = destination.name.split(".");
      const destinationNodeId = parseInt(destinationSocketStr[0]);
      const destinationSocketId = parseInt(destinationSocketStr[1]);

      const sourceSocket = await outputSocketById(sourceSocketId);
      const sourceProviderId = sourceSocket.provider.id;

      const destinationSocket = await inputSocketById(destinationSocketId);
      const destinationProviderId = destinationSocket.provider.id;

      if (_.isEqual(sourceSocket.provider.ty, destinationSocket.provider.ty)) {
        sceneManager.createConnection(
          {
            x:
              (source.worldTransform.tx - this.state.offset.x) *
              (1 / this.state.zoomFactor),
            y:
              (source.worldTransform.ty - this.state.offset.y) *
              (1 / this.state.zoomFactor),
          },
          {
            x:
              (destination.worldTransform.tx - this.state.offset.x) *
              (1 / this.state.zoomFactor),
            y:
              (destination.worldTransform.ty - this.state.offset.y) *
              (1 / this.state.zoomFactor),
          },
          source.name,
          destination.name,
          sourceSocket.provider.color,
        );

        this.state.createdConnection = true;
        this.clearInteractiveConnection(sceneManager);
        sceneManager.refreshConnections();

        if (sourceNodeId !== -1 && destinationNodeId !== -1) {
          this.dataManager.createConnection({
            sourceNodeId,
            sourceSocketId,
            sourceProviderId,
            destinationNodeId,
            destinationSocketId,
            destinationProviderId,
          });
        }
      }
    }
  }

  clearInteractiveConnection(sceneManager: SceneManager): void {
    const nodes = sceneManager.group.nodes.children as Node[];
    for (const node of nodes) {
      node.undim();
    }

    if (
      this.state &&
      !this.state.targetWasConnected &&
      !this.state.createdConnection
    ) {
      this.state.target.setDisconnected();
    }
    if (sceneManager.interactiveConnection) {
      sceneManager.removeConnection(sceneManager.interactiveConnection.name);
    }
    this.state = undefined;
  }
}

interface Position {
  x: number;
  y: number;
}
