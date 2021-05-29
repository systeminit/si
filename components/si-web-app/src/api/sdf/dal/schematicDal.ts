import { EdgeKind, Edge } from "@/api/sdf/model/edge";
import { Entity } from "@/api/sdf/model/entity";
import { System } from "@/api/sdf/model/system";
import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";
import {
  Schematic,
  ISchematicNode,
  SchematicKind,
} from "@/api/sdf/model/schematic";

export interface IGetSchematicRequest {
  workspaceId: string;
  rootObjectId: string;
  changeSetId?: string;
  editSessionId?: string;
  includeRootNode: boolean;
  schematicKind: SchematicKind;
}

export interface IGetSchematicReplySuccess {
  schematic: Schematic;
  error?: never;
}

export interface IGetSchematicReplyFailure {
  schematic?: never;
  error: SDFError;
}

export type IGetSchematicReply =
  | IGetSchematicReplySuccess
  | IGetSchematicReplyFailure;

export interface IGetApplicationSystemSchematicRequest
  extends IGetSchematicRequest {
  systemId: string;
}

export async function getApplicationSystemSchematic(
  request: IGetApplicationSystemSchematicRequest,
): Promise<IGetSchematicReply> {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;

  const reply: IGetSchematicReply = await sdf.get(
    "schematicDal/getApplicationSystemSchematic",
    request,
  );
  return reply;
}

/*
 * Connections
 */

export interface ConnectionNodeReference {
  nodeId: string;
  socketId: string;
  socketName?: string;
  nodeKind: string;
}

export interface Connection {
  source: ConnectionNodeReference;
  destination: ConnectionNodeReference;
}

export interface ConnectionCreateRequest {
  connection: Connection;
  workspaceId: string;
  changeSetId: string;
  editSessionId: string;
  rootObjectId: string;
  schematicKind: SchematicKind;
}

export interface ConnectionCreateReplySuccess {
  edge: Edge;
  schematic?: never;
  error?: never;
}

export interface ConnectionCreateReplySuccessWithSchematic {
  edge: Edge;
  schematic: Schematic;
  error?: never;
}

export interface ConnectionCreateReplyFailure {
  edge?: Edge;
  schematic?: never;
  error: SDFError;
}

export type ConnectionCreateReply =
  | ConnectionCreateReplySuccess
  | ConnectionCreateReplySuccessWithSchematic
  | ConnectionCreateReplyFailure;

async function connectionCreate(
  request: ConnectionCreateRequest,
): Promise<ConnectionCreateReply> {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;

  const reply: ConnectionCreateReply = await sdf.post(
    "schematicDal/connectionCreate",
    request,
  );
  return reply;
}

/*
 * Nodes
 */

export interface INodeCreateForApplicationRequest extends INodeCreateRequest {
  applicationId: string;
  deploymentSelectedEntityId?: string;
}

export interface INodeCreateRequest {
  name?: string;
  entityType: string;
  workspaceId: string;
  changeSetId: string;
  editSessionId: string;
  schematicKind: SchematicKind;
}

export interface INodeCreateReplySuccess {
  node: ISchematicNode;
  schematic?: never;
  error?: never;
}

export interface INodeCreateReplySuccessWithSchematic {
  node: ISchematicNode;
  schematic: Schematic;
  error?: never;
}

export interface INodeCreateReplyFailure {
  node?: never;
  schematic?: never;
  error: SDFError;
}

export type INodeCreateReply =
  | INodeCreateReplySuccess
  | INodeCreateReplySuccessWithSchematic
  | INodeCreateReplyFailure;

export interface INodeObjectEntity {
  entity: Entity;
  system?: never;
}

export interface INodeObjectSystem {
  entity?: never;
  system: System;
}

export type INodeObject = INodeObjectEntity | INodeObjectSystem;

export interface INodeUpdatePositionReplySuccess {
  error?: never;
}

export interface INodeUpdatePositionReplyFailure {
  error: SDFError;
}

export type INodeUpdatePositionReply =
  | INodeUpdatePositionReplySuccess
  | INodeUpdatePositionReplyFailure;

export interface INodeUpdatePositionRequest {
  nodeId: string;
  contextId: string;
  x: string;
  y: string;
  workspaceId: string;
}

export interface INodeDeleteRequest {
  nodeId: string;
  applicationId: string;
  workspaceId: string;
  changeSetId: string;
  editSessionId: string;
  systemId: string;
}

export interface INodeDeleteReplySuccess {
  deleted: boolean;
  error?: never;
}

export interface INodeDeleteReplyFailure {
  deleted?: never;
  error: SDFError;
}

export type INodeDeleteReply =
  | INodeDeleteReplySuccess
  | INodeDeleteReplyFailure;

async function nodeCreateForApplication(
  request: INodeCreateForApplicationRequest,
): Promise<INodeCreateReply> {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;

  const reply: INodeCreateReply = await sdf.post(
    "schematicDal/nodeCreateForApplication",
    request,
  );
  return reply;
}

async function nodeUpdatePosition(
  request: INodeUpdatePositionRequest,
): Promise<INodeUpdatePositionReply> {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;

  const reply: INodeUpdatePositionReply = await sdf.post(
    "schematicDal/updateNodePosition",
    request,
  );
  return reply;
}

async function nodeDelete(
  request: INodeDeleteRequest,
): Promise<INodeDeleteReply> {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;

  const reply: INodeDeleteReply = await sdf.post(
    "schematicDal/deleteNode",
    request,
  );
  return reply;
}

export const SchematicDal = {
  getApplicationSystemSchematic,
  connectionCreate,
  nodeCreateForApplication,
  nodeUpdatePosition,
  nodeDelete,
};
