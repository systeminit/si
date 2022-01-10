import * as PIXI from "pixi.js";
import * as PIXIFILTER from "pixi-filters";

import * as MODEL from "../../model";

import _ from "lodash";

import { Connection } from "./connection";
import { NodeTitle } from "./node/nodeTitle";
import { NodeName } from "./node/nodeName";
import { Socket, SocketType } from "./node/socket";

interface Position {
  x: number;
  y: number;
}

export interface NodeData {
  name: string;
  position: Position;
}

const NODE_WIDTH = 140;
const NODE_HEIGHT = 100;

const INPUT_SOCKET_OFFSET = 45;
const OUTPUT_SOCKET_OFFSET = 35;
const SOCKET_SPACING = 20;
const SOCKET_HEIGHT = 3;

export class Node extends PIXI.Container {
  kind: string;
  isSelected = false;
  id: string;
  title: string;
  connections: Array<Connection>;
  selection?: PIXI.Graphics;

  constructor(n: MODEL.Node) {
    super();
    this.id = n.id;

    this.name = n.label.name;
    this.title = n.label.title;
    this.kind = "node";
    this.connections = [];

    // TODO: we should get the propper function for our context instead of assuming it's the first one
    if (n.position.length > 0) {
      this.x = n.position[0].x;
      this.y = n.position[0].y;
    }
    this.interactive = true;
    this.buttonMode = true;
    this.sortableChildren = true;

    // Card object
    this.setCard(Math.max(n.input.length, n.output.length));
    this.setTitle();
    this.setName();

    // Sockets
    this.setInputSockets(n.input);
    this.setOutputSockets(n.output);

    // Selection hilight
    this.setSelection(Math.max(n.input.length, n.output.length));

    // Shadow
    this.setShadows();
  }

  setCard(socketCount: number): void {
    const card = new PIXI.Graphics()
      .beginFill(0x282e30)
      .drawRoundedRect(0, 0, NODE_WIDTH, this.nodeHeight(socketCount), 6)
      .endFill();
    card.zIndex = 1;
    this.addChild(card);
  }

  setSelection(socketCount: number): void {
    this.selection = new PIXI.Graphics()
      .lineStyle(1, 0x4dfaff, 1, 0, false)
      .drawRoundedRect(0, 0, NODE_WIDTH, this.nodeHeight(socketCount), 6);
    this.selection.zIndex = 2;
    this.addChild(this.selection);
    this.deselect();
  }

  setShadows(): void {
    const dropShadow = new PIXIFILTER.DropShadowFilter();
    dropShadow.color = 0x000000;
    dropShadow.blur = 1;
    dropShadow.distance = 2;
    dropShadow.quality = 3;
    dropShadow.alpha = 0.5;
    dropShadow.resolution = window.devicePixelRatio || 1;
    this.filters = [dropShadow];
  }

  setTitle(): void {
    const title = new NodeTitle(this.title, NODE_WIDTH);
    this.addChild(title);
  }

  setName(): void {
    const name = new NodeName(this.name, NODE_WIDTH);
    this.addChild(name);
  }

  setInputSockets(inputs: MODEL.Socket[]): void {
    for (let i = 0; i < inputs.length; i++) {
      const s = inputs[i];

      let socketLabel = null;
      if (s.name) {
        socketLabel = s.name;
      }
      const socket = new Socket(
        s.id,
        socketLabel,
        SocketType.input,
        {
          x: 0,
          y: INPUT_SOCKET_OFFSET + SOCKET_SPACING * i,
        },
        0xffdd44,
      );
      socket.zIndex = 3;
      this.addChild(socket);
    }
  }

  setOutputSockets(outputs: MODEL.Socket[]): void {
    for (let i = 0; i < outputs.length; i++) {
      const s = outputs[i];

      const socket = new Socket(
        s.id,
        null,
        SocketType.output,
        {
          x: NODE_WIDTH,
          y: OUTPUT_SOCKET_OFFSET,
        },
        0xeb44ff,
      );
      socket.zIndex = 3;
      this.addChild(socket);
    }
  }

  select(): void {
    this.isSelected = true;
    this.selection.visible = true;
  }

  deselect(): void {
    this.isSelected = false;
    this.selection.visible = false;
  }

  addConnection(c: Connection): void {
    this.connections.push(c);
  }

  nodeHeight(socketCount: number): number {
    const height =
      INPUT_SOCKET_OFFSET +
      (SOCKET_HEIGHT + SOCKET_SPACING) * socketCount -
      SOCKET_SPACING * 0.65;
    return Math.max(height, NODE_HEIGHT);
  }
}
