import * as PIXI from "pixi.js";
import * as PIXIFILTER from "pixi-filters";

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

const NODE_WIDTH = 140;
const NODE_HEIGHT = 100;

const INPUT_SOCKET_OFFSET = 45;
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

    // Ignores fake nodes as we might not have the schema variants for now (the backend isn't there yet)
    // The v.id > 0 should go away eventually
    if (v.id > 0 && v.id !== n.schemaVariantId)
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
    this.setCard(size);

    // Selection status
    this.setSelectionStatus(size);

    this.setSockets(v.inputSockets, v.outputSockets);

    this.setQualificationStatus(undefined);
    this.setResourceStatus(undefined);

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

  setQualificationStatus(qualified?: boolean): void {
    const oldStatus = this.getChildByName("QualificationStatus");

    const status = new QualificationStatus(qualified);
    status.name = "QualificationStatus";
    status.zIndex = 1;
    status.x = 100;
    status.y = 78;
    this.addChild(status);

    oldStatus?.destroy();
  }

  setResourceStatus(health?: ResourceHealth): void {
    const oldStatus = this.getChildByName("ResourceStatus");

    const status = new ResourceStatus(health);
    status.name = "ResourceStatus";
    status.zIndex = 1;
    status.x = 120;
    status.y = 78;
    this.addChild(status);

    oldStatus?.destroy();
  }

  setSockets(
    inputs: SchematicInputSocket[],
    outputs: SchematicOutputSocket[],
  ): void {
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

  nodeHeight(socketCount: number): number {
    const height =
      INPUT_SOCKET_OFFSET +
      (SOCKET_HEIGHT + SOCKET_SPACING) * socketCount -
      SOCKET_SPACING * 0.65;
    return Math.max(height, NODE_HEIGHT);
  }
}
