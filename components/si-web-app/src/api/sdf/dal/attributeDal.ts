import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";
import { Entity } from "@/api/sdf/model/entity";
import { Diff } from "../model/diff";
import { Qualification } from "@/api/sdf/model/qualification";
import { ILabelList } from "../dal";
import { SchematicKind } from "../model/schematic";

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
  qualifications: Qualification[];
  error?: never;
}

export interface IGetEntityReplyFailure {
  entity?: never;
  diff?: never;
  qualifications?: never;
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
  systemId?: string;
}

export interface IUpdateEntityReplySuccess {
  entity: Entity;
  diff: Diff;
  label: { label: string; value: string };
  qualifications: Qualification[];
  error?: never;
}

export interface IUpdateEntityReplyFailure {
  entity?: never;
  diff?: never;
  label?: never;
  qualifications?: never;
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

export interface IGetInputLabelsRequest {
  workspaceId: string;
  entityId: string;
  inputName: string;
  schematicKind: SchematicKind;
  changeSetId?: string;
  editSessionId?: string;
}

export interface IGetInputLabelsReplySuccess {
  items: ILabelList;
  error?: never;
}

export interface IGetInputLabelsReplyFailure {
  items?: never;
  error: SDFError;
}

export type IGetInputLabelsReply =
  | IGetInputLabelsReplySuccess
  | IGetInputLabelsReplyFailure;

export async function getInputLabels(
  request: IGetInputLabelsRequest,
): Promise<IGetInputLabelsReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IGetInputLabelsReply = await sdf.get(
    "attributeDal/getInputLabels",
    request,
  );
  return reply;
}

export const AttributeDal = {
  getEntityList,
  getEntity,
  updateEntity,
  getInputLabels,
};
