import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";
import { Resource } from "@/api/sdf/model/resource";

export interface IGetResourceRequest {
  entityId: string;
  systemId: string;
  workspaceId: string;
}

export interface IGetResourceReplySuccess {
  resource: Resource | null;
  error?: never;
}

export interface IGetResourceReplyFailure {
  resource?: never;
  error: SDFError;
}

export type IGetResourceReply =
  | IGetResourceReplySuccess
  | IGetResourceReplyFailure;

export async function getResource(
  request: IGetResourceRequest,
): Promise<IGetResourceReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IGetResourceReply = await sdf.get(
    "resourceDal/getResource",
    request,
  );
  return reply;
}

export interface ISyncResourceRequest {
  entityId: string;
  systemId: string;
  workspaceId: string;
}

export interface ISyncResourceReplySuccess {
  started: boolean;
  error?: never;
}

export interface ISyncResourceReplyFailure {
  started?: never;
  error: SDFError;
}

export type ISyncResourceReply =
  | ISyncResourceReplySuccess
  | ISyncResourceReplyFailure;

export async function syncResource(
  request: ISyncResourceRequest,
): Promise<ISyncResourceReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: ISyncResourceReply = await sdf.post(
    "resourceDal/syncResource",
    request,
  );
  return reply;
}

export const ResourceDal = {
  getResource,
  syncResource,
};
