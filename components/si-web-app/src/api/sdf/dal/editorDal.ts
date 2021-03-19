import { Node } from "@/api/sdf/model/node";
import { Entity } from "@/api/sdf/model/entity";
import { SDFError } from "@/api/sdf";
import { System } from "@/api/sdf/model/system";
import Bottle from "bottlejs";

export interface INodeCreateForApplicationRequest extends INodeCreateRequest {
  applicationId: string;
}

export interface INodeCreateRequest {
  name?: string;
  entityType: string;
  workspaceId: string;
  changeSetId: string;
  editSessionId: string;
}

export interface INodeUpdatePositionRequest {
  nodeId: string;
  contextId: string;
  x: string;
  y: string;
  workspaceId: string;
}

export interface INodeObjectEntity {
  entity: Entity;
  system?: never;
}

export interface INodeObjectSystem {
  entity?: never;
  system: System;
}

export type INodeObject = INodeObjectEntity | INodeObjectSystem;

export interface INodeCreateReplySuccess {
  node: Node;
  object: INodeObject;
  error?: never;
}

export interface INodeCreateReplyFailure {
  node?: never;
  object?: never;
  error: SDFError;
}

export interface INodeUpdatePositionReplySuccess {
  // nodePosition: any; // ignoring this for now.
  error?: never;
}

export interface INodeUpdatePositionReplyFailure {
  // nodePosition?: any; // ignoring this for now.
  error: SDFError;
}

export type INodeCreateReply =
  | INodeCreateReplySuccess
  | INodeCreateReplyFailure;

export type INodeUpdatePositionReply =
  | INodeUpdatePositionReplySuccess
  | INodeUpdatePositionReplyFailure;

async function nodeCreateForApplication(
  request: INodeCreateForApplicationRequest,
): Promise<INodeCreateReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: INodeCreateReply = await sdf.post(
    "editorDal/nodeCreateForApplication",
    request,
  );
  return reply;
}
async function nodeUpdatePosition(
  request: INodeUpdatePositionRequest,
): Promise<INodeUpdatePositionReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: INodeUpdatePositionReply = await sdf.post(
    "editorDal/updateNodePosition",
    request,
  );
  return reply;
}

export interface IEntitySetPropertyRequest {
  workspaceId: string;
  entityId: string;
  changeSetId: string;
  editSessionId: string;
  overrideSystem?: string;
  path: string[];
  value: any;
}

export interface IEntitySetPropertyReplySuccess {
  object: Entity;
  error?: never;
}

export interface IEntitySetPropertyReplyFailure {
  object?: never;
  error: SDFError;
}

export type IEntitySetPropertyReply =
  | IEntitySetPropertyReplySuccess
  | IEntitySetPropertyReplyFailure;

async function entitySetProperty(
  request: IEntitySetPropertyRequest,
): Promise<IEntitySetPropertyReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IEntitySetPropertyReply = await sdf.post(
    "editorDal/entitySetProperty",
    request,
  );
  return reply;
}

export interface IEntitySetPropertyBulkRequest {
  workspaceId: string;
  entityId: string;
  changeSetId: string;
  editSessionId: string;
  overrideSystem?: string;
  properties: {
    path: string[];
    value: any;
  }[];
}

export interface IEntitySetPropertyBulkReplySuccess {
  object: Entity;
  error?: never;
}

export interface IEntitySetPropertyBulkReplyFailure {
  object?: never;
  error: SDFError;
}

export type IEntitySetPropertyBulkReply =
  | IEntitySetPropertyBulkReplySuccess
  | IEntitySetPropertyBulkReplyFailure;

async function entitySetPropertyBulk(
  request: IEntitySetPropertyBulkRequest,
): Promise<IEntitySetPropertyBulkReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IEntitySetPropertyBulkReply = await sdf.post(
    "editorDal/entitySetPropertyBulk",
    request,
  );
  return reply;
}

export interface IEntitySetNameRequest {
  workspaceId: string;
  entityId: string;
  changeSetId: string;
  editSessionId: string;
  overrideSystem?: string;
  name: string;
}

export interface IEntitySetNameReplySuccess {
  object: Entity;
  error?: never;
}

export interface IEntitySetNameReplyFailure {
  object?: never;
  error: SDFError;
}

export type IEntitySetNameReply =
  | IEntitySetNameReplySuccess
  | IEntitySetNameReplyFailure;

async function entitySetName(
  request: IEntitySetNameRequest,
): Promise<IEntitySetNameReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IEntitySetNameReply = await sdf.post(
    "editorDal/entitySetName",
    request,
  );
  return reply;
}

export const EditorDal = {
  nodeCreateForApplication,
  entitySetProperty,
  entitySetPropertyBulk,
  entitySetName,
  nodeUpdatePosition,
};
