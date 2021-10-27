import { Entity } from "@/api/sdf/model/entity";
import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";

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
  entitySetProperty,
  entitySetPropertyBulk,
  entitySetName,
};
