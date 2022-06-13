import * as PIXI from "pixi.js";

import _ from "lodash";

import { Card } from "./node/card";
import { Sockets } from "./node/sockets";
import { Connection } from "./connection";
import { SelectionStatus } from "./node/status";
import { QualificationStatus } from "./node/status";
import { ResourceStatus } from "./node/status/resource";
import { SchematicKind, SchematicNodeKind } from "@/api/sdf/dal/schematic";
import { ResourceHealth } from "@/api/sdf/dal/resource";
import {
  SchematicNode,
  SchematicSchemaVariant,
  SchematicInputSocket,
  SchematicOutputSocket,
} from "@/api/sdf/dal/schematic";

const NODE_WIDTH = 160;
const NODE_HEIGHT = 120;

const INPUT_SOCKET_OFFSET = 60;
// const OUTPUT_SOCKET_OFFSET = 35;
const SOCKET_SPACING = 20;
const SOCKET_HEIGHT = 3;

interface Position {
  x: number;
  y: number;
}

export class Node extends PIXI.Container {
  id: number;
  kind: string;
  schemaVariantId: number;
  color: number;
  nodeKind: SchematicNodeKind;
  isSelected = false;
  title: string;
  connections: Array<Connection>;
  panelKind: SchematicKind;
  selection?: SelectionStatus;
  socketCount: number;

  constructor(
    n: SchematicNode,
    v: SchematicSchemaVariant,
    pos: Position,
    panelKind: SchematicKind,
  ) {
    super();
    this.id = n.id;
    this.color = v.color;

    this.panelKind = panelKind;

    if (v.id !== n.schemaVariantId)
      throw new Error("mismatch in schema variants");

    this.schemaVariantId = n.schemaVariantId;

    this.name = n.name;
    this.title = n.title;
    this.kind = "node";
    this.nodeKind = n.kind;
    this.connections = [];

    this.x = pos.x;
    this.y = pos.y;

    this.interactive = true;
    this.buttonMode = true;
    this.sortableChildren = true;

    // Card object
    const size = Math.max(v.inputSockets.length, v.outputSockets.length);
    this.socketCount = size;
    this.setCard();

    // Selection status
    this.setSelectionStatus();

    this.setSockets(v.inputSockets, v.outputSockets);

    this.setQualificationStatus(null);
    this.setResourceStatus(null);
  }

  setCard(): void {
    const card = new Card(
      NODE_WIDTH,
      this.nodeHeight(),
      6,
      this.title,
      this.name,
      this.color,
    );
    card.zIndex = 0;
    this.addChild(card);
  }

  undim(): void {
    this.alpha = 1;
  }

  dim(): void {
    this.alpha = 0.3;
  }

  setSelectionStatus(): void {
    const status = new SelectionStatus(
      NODE_WIDTH,
      this.nodeHeight(),
      6,
      this.color,
    );
    status.zIndex = 1;
    this.selection = status;
    this.addChild(this.selection);
    this.deselect();
  }

  setQualificationStatus(qualified: boolean | null): void {
    const oldStatus = this.getChildByName("QualificationStatus");

    const status = new QualificationStatus(
      qualified,
      118,
      98,
      NODE_WIDTH,
      this.nodeHeight(),
    );
    status.name = "QualificationStatus";
    status.zIndex = 1;
    this.addChild(status);

    oldStatus?.destroy();
  }

  setResourceStatus(health: ResourceHealth | null): void {
    const oldStatus = this.getChildByName("ResourceStatus");

    const status = new ResourceStatus(health);
    status.name = "ResourceStatus";
    status.zIndex = 1;
    status.x = 138;
    status.y = 98;
    this.addChild(status);

    oldStatus?.destroy();
  }

  setSockets(
    inputs: SchematicInputSocket[],
    outputs: SchematicOutputSocket[],
  ): void {
    const sockets = new Sockets(this.id, inputs, outputs, this.panelKind);
    sockets.name = "Sockets";
    sockets.zIndex = 2;
    this.addChild(sockets);
  }

  select(): void {
    if (!this.isSelected) this.zIndex += 1;
    this.isSelected = true;
    if (this.selection) this.selection.visible = true;
  }

  deselect(): void {
    if (this.isSelected) this.zIndex -= 1;
    this.isSelected = false;
    if (this.selection) this.selection.visible = false;
  }

  addConnection(c: Connection): void {
    this.connections.push(c);
  }

  nodeHeight(): number {
    const height =
      INPUT_SOCKET_OFFSET +
      (SOCKET_HEIGHT + SOCKET_SPACING) * this.socketCount -
      SOCKET_SPACING * 0.65;
    return Math.max(height, NODE_HEIGHT);
  }
}
