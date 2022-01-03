import * as PIXI from "pixi.js";
import * as PIXIFILTER from "pixi-filters";

import * as MODEL from "../../model";

import _ from "lodash";

import { Socket, SocketType } from "./socket";
import { Connection } from "./connection";

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
  selection: PIXI.Graphics;
  isSelected = false;
  id: string;
  title: string;
  connections: Array<Connection>;
  // interaction: Interaction;

  constructor(n: MODEL.Node) {
    super();
    this.id = n.id;

    this.name = n.label.name;
    this.title = n.label.title;
    this.kind = "node";
    this.connections = [];

    this.x = n.position.ctx[0].position.x;
    this.y = n.position.ctx[0].position.y;
    this.interactive = true;
    this.buttonMode = true;
    this.sortableChildren = true;

    // Shadow
    const dropShadow = new PIXIFILTER.DropShadowFilter();
    dropShadow.color = 0x000000;
    dropShadow.blur = 1;
    dropShadow.distance = 2;
    dropShadow.quality = 3;
    dropShadow.alpha = 0.5;
    dropShadow.resolution = window.devicePixelRatio || 1;
    this.filters = [dropShadow];

    this.selection = new PIXI.Graphics()
      .lineStyle(1, 0x4dfaff, 1, 0, false)
      .drawRoundedRect(0, 0, NODE_WIDTH, this.nodeHeight(n.input.length), 6);
    this.selection.zIndex = 2;

    this.addChild(this.selection);
    this.deselect();

    // Card object
    const card = new PIXI.Graphics()
      .beginFill(0x282e30)
      .drawRoundedRect(0, 0, NODE_WIDTH, this.nodeHeight(n.input.length), 6)
      .endFill();
    card.zIndex = 1;
    this.addChild(card);

    // Create input sockets
    for (let i = 0; i < n.input.length; i++) {
      const s = n.input[i];

      let socketLabel = null;
      if (s.name) {
        socketLabel = s.name;
      }

      const socket = this.createSocket(
        s.id,
        socketLabel,
        SocketType.input,
        {
          x: 0,
          y: INPUT_SOCKET_OFFSET + SOCKET_SPACING * i,
        },
        0xffdd44,
      );
      this.addSocket(socket);
    }

    for (let i = 0; i < n.output.length; i++) {
      const s = n.output[i];

      let socketName = null;
      if (s.name) {
        socketName = s.name;
      }
      const socket = this.createSocket(
        s.id,
        socketName,
        SocketType.output,
        {
          x: NODE_WIDTH,
          y: OUTPUT_SOCKET_OFFSET,
        },
        0xeb44ff,
      );
      this.addSocket(socket);
    }

    // Title
    const title = new PIXI.Text(this.title, {
      fontFamily: "Source Code Pro",
      fontSize: 12,
      fontWeight: "400",
      letterSpacing: 0,
      fill: "white",
      align: "left",
    });
    title.position.x = NODE_WIDTH * 0.5 - title.width * 0.5;
    title.position.y = 5;
    title.zIndex = 2;
    this.addChild(title);

    // Name
    const name = new PIXI.Text(this.name, {
      fontFamily: "Source Code Pro",
      fontSize: 10,
      fontWeight: "400",
      letterSpacing: 0,
      fill: "white",
      align: "left",
    });
    name.position.x = NODE_WIDTH * 0.5 - name.width * 0.5;
    name.position.y = 25;
    name.zIndex = 2;
    this.addChild(name);
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

  createSocket(
    id: string,
    label: string | null,
    type: SocketType,
    position: Position,
    color: number,
  ): Socket {
    const socket = new Socket(
      id,
      label,
      type,
      {
        x: position.x,
        y: position.y,
      },
      color,
    );
    socket.zIndex = 2;
    return socket;
  }

  addSocket(s: Socket): void {
    this.addChild(s);
  }

  nodeHeight(socketCount: number): number {
    const height =
      INPUT_SOCKET_OFFSET +
      (SOCKET_HEIGHT + SOCKET_SPACING) * socketCount -
      SOCKET_SPACING * 0.65;
    return Math.max(height, NODE_HEIGHT);
  }
}
