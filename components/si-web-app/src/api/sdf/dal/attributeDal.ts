import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";
import { Entity } from "@/api/sdf/model/entity";

export interface IGetObjectListRequest {
  workspaceId: string;
  applicationId: string;
  changeSetId?: string;
}

export interface IGetObjectListReplySuccess {
  objectList: { label: string; value: string }[];
  error?: never;
}

export interface IGetObjectListReplyFailure {
  objectList?: never;
  error: SDFError;
}

export type IGetObjectListReply =
  | IGetObjectListReplySuccess
  | IGetObjectListReplyFailure;

export async function getObjectList(
  request: IGetObjectListRequest,
): Promise<IGetObjectListReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IGetObjectListReply = await sdf.get(
    "attributeDal/getObjectList",
    request,
  );
  return reply;
}

export interface IGetEntityRequest {
  workspaceId: string;
  entityId: string;
  changeSetId?: string;
}

export interface IGetEntityReplySuccess {
  entity: Entity;
  baseEntity: Entity;
  error?: never;
}

export interface IGetEntityReplyFailure {
  entity?: never;
  baseEntity?: never;
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

export const AttributeDal = {
  getObjectList,
  getEntity,
};
