import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";
import { Entity } from "@/api/sdf/model/entity";
import { Diff } from "../model/diff";

export interface IGetEntityListRequest {
  workspaceId: string;
  applicationId: string;
  changeSetId?: string;
  editSessionId?: string;
}

export interface IGetEntityListReplySuccess {
  entityList: { label: string; value: string }[];
  error?: never;
}

export interface IGetEntityListReplyFailure {
  objectList?: never;
  error: SDFError;
}

export type IGetEntityListReply =
  | IGetEntityListReplySuccess
  | IGetEntityListReplyFailure;

export async function getEntityList(
  request: IGetEntityListRequest,
): Promise<IGetEntityListReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IGetEntityListReply = await sdf.get(
    "attributeDal/getEntityList",
    request,
  );
  return reply;
}

export interface IGetEntityRequest {
  workspaceId: string;
  entityId: string;
  changeSetId?: string;
  editSessionId?: string;
}

export interface IGetEntityReplySuccess {
  entity: Entity;
  diff: Diff;
  error?: never;
}

export interface IGetEntityReplyFailure {
  entity?: never;
  diff?: never;
  error: SDFError;
}

export type IGetEntityReply = IGetEntityReplySuccess | IGetEntityReplyFailure;

export async function getEntity(
  request: IGetEntityRequest,
): Promise<IGetEntityReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IGetEntityReply = await sdf.get(
    "attributeDal/getEntity",
    request,
  );
  return reply;
}

export interface IUpdateEntityRequest {
  workspaceId: string;
  entity: Entity;
  changeSetId: string;
  editSessionId: string;
}

export interface IUpdateEntityReplySuccess {
  entity: Entity;
  diff: Diff;
  label: { label: string; value: string };
  error?: never;
}

export interface IUpdateEntityReplyFailure {
  entity?: never;
  diff?: never;
  label?: never;
  error: SDFError;
}

export type IUpdateEntityReply =
  | IUpdateEntityReplySuccess
  | IUpdateEntityReplyFailure;

export async function updateEntity(
  request: IUpdateEntityRequest,
): Promise<IUpdateEntityReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IUpdateEntityReply = await sdf.post(
    "attributeDal/updateEntity",
    request,
  );
  return reply;
}

export const AttributeDal = {
  getEntityList,
  getEntity,
  updateEntity,
};
