import * as PIXI from "pixi.js";
import * as PIXIFILTER from "pixi-filters";

import * as MODEL from "../../model";

import _ from "lodash";

import { Card } from "./node/card";
import { Sockets } from "./node/sockets";
import { Connection } from "./connection";
import { SelectionStatus } from "./node/status";
import { QualificationStatus } from "./node/status";
import { ResourceStatus } from "./node/status/resource";
import { SchematicKind } from "@/api/sdf/dal/schematic";
import { NodeKind } from "@/api/sdf/dal/node";

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
// const OUTPUT_SOCKET_OFFSET = 35;
const SOCKET_SPACING = 20;
const SOCKET_HEIGHT = 3;

export class Node extends PIXI.Container {
  id: number;
  kind: string;
  nodeKind?: { kind: NodeKind; componentId?: number };
  isSelected = false;
  title: string;
  connections: Array<Connection>;
  panelKind: SchematicKind;
  selection?: SelectionStatus;

  constructor(n: MODEL.Node, pos: Position, panelKind: SchematicKind) {
    super();
    this.id = n.id;

    this.panelKind = panelKind;

    this.name = n.label.name;
    this.title = n.label.title;
    this.kind = "node";
    this.nodeKind = n.kind;
    this.connections = [];

    this.x = pos.x;
    this.y = pos.y;

    this.interactive = true;
    this.buttonMode = true;
    this.sortableChildren = true;

    // Card object
    this.setCard(Math.max(n.input.length, n.output.length));
    this.setSockets(n.input, n.output);

    // Selection status
    this.setSelectionStatus(Math.max(n.input.length, n.output.length));

    this.setQualificationStatus();

    this.setResourceStatus();

    // Shadow
    this.setShadows();
  }

  setCard(socketCount: number): void {
    const card = new Card(
      NODE_WIDTH,
      this.nodeHeight(socketCount),
      6,
      this.title,
      this.name,
    );
    card.zIndex = 0;
    this.addChild(card);
  }

  setSelectionStatus(socketCount: number): void {
    const status = new SelectionStatus(
      NODE_WIDTH,
      this.nodeHeight(socketCount),
      6,
    );
    status.zIndex = 1;
    this.selection = status;
    this.addChild(this.selection);
    this.deselect();
  }

  setQualificationStatus(): void {
    const status = new QualificationStatus();
    status.x = 100;
    status.y = 78;
    this.addChild(status);
  }

  setResourceStatus(): void {
    const status = new ResourceStatus();
    status.x = 120;
    status.y = 78;
    this.addChild(status);
  }

  setSockets(inputs: MODEL.Socket[], outputs: MODEL.Socket[]): void {
    const sockets = new Sockets(this.id, inputs, outputs, this.panelKind);
    sockets.zIndex = 2;
    this.addChild(sockets);
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

  select(): void {
    this.isSelected = true;
    if (this.selection) this.selection.visible = true;
  }

  deselect(): void {
    this.isSelected = false;
    if (this.selection) this.selection.visible = false;
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
