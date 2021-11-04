import * as PIXI from "pixi.js";
import * as PIXIFILTER from "pixi-filters";

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

const INPUT_SOCKET_OFFSET = NODE_HEIGHT * 0.5;
const OUTPUT_SOCKET_OFFSET = NODE_HEIGHT * 0.5;

export class Node extends PIXI.Container {
  name: string;
  kind: string;
  selection: PIXI.Graphics;
  isSelected = false;
  id: string;
  connections: Array<Connection>;
  // interaction: Interaction;

  constructor(name: string, position: Position) {
    super();

    this.id = _.uniqueId();
    this.name = name;
    this.kind = "node";
    this.connections = [];

    this.x = position.x;
    this.y = position.y;
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
      .drawRoundedRect(0, 0, NODE_WIDTH, NODE_HEIGHT, 6);
    this.selection.zIndex = 2;

    this.addChild(this.selection);
    this.deselect();

    // Card object
    const card = new PIXI.Graphics()
      .beginFill(0x00979b)
      .drawRoundedRect(0, 0, NODE_WIDTH, NODE_HEIGHT, 6)
      .endFill();
    card.zIndex = 1;
    this.addChild(card);

    // Input socket
    let socket = new Socket(
      this.name + ".inputSocket",
      SocketType.input,
      {
        x: 0,
        y: INPUT_SOCKET_OFFSET,
      },
      0xffdd44,
    );
    socket.zIndex = 2;
    this.addChild(socket);

    // Output socket
    socket = new Socket(
      this.name + ".outputSocket",
      SocketType.output,
      { x: NODE_WIDTH, y: OUTPUT_SOCKET_OFFSET },
      0xeb44ff,
    );
    socket.zIndex = 2;
    this.addChild(socket);

    // Title
    const title = new PIXI.Text(this.name, {
      fontFamily: "Source Code Pro",
      fontSize: 12,
      fill: "white",
      align: "left",
    });
    title.position.x = NODE_WIDTH * 0.5 - title.width * 0.5;
    title.position.y = 5;
    title.zIndex = 2;
    this.addChild(title);
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
}
