import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";
import { Entity } from "@/api/sdf/model/entity";
import { Diff } from "../model/diff";
import { Qualification } from "@/api/sdf/model/qualification";
import { ILabelList } from "../dal";
import { SchematicKind } from "../model/schematic";
import { Connection } from "../model/connection";
import { Resource } from "si-entity";

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

export interface IGetDiscoveryListRequest {
  workspaceId: string;
  entityType: string;
}

export interface IGetDiscoveryListReplySuccess {
  list: { entity: Entity; resource: Resource }[];
  error?: never;
}

export interface IGetDiscoveryListReplyFailure {
  objectList?: never;
  error: SDFError;
}

export type IGetDiscoveryListReply =
  | IGetDiscoveryListReplySuccess
  | IGetDiscoveryListReplyFailure;

export async function getDiscoveryList(
  request: IGetDiscoveryListRequest,
): Promise<IGetDiscoveryListReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IGetDiscoveryListReply = await sdf.get(
    "attributeDal/getDiscoveryList",
    request,
  );
  return reply;
}

export interface IGetImplementationsListRequest {
  workspaceId: string;
  applicationId: string;
  implementationEntityTypes: string[];
}

export interface IGetImplementationsListReplySuccess {
  list: {
    [entityType: string]: { entity: Entity; resource: Resource }[];
  };
  error?: never;
}

export interface IGetImplementationsListReplyFailure {
  objectList?: never;
  error: SDFError;
}

export type IGetImplementationsListReply =
  | IGetImplementationsListReplySuccess
  | IGetImplementationsListReplyFailure;

export async function getImplementationsList(
  request: IGetImplementationsListRequest,
): Promise<IGetImplementationsListReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IGetImplementationsListReply = await sdf.post(
    "attributeDal/getImplementationsList",
    request,
  );
  return reply;
}

export interface IDiscoverRequest {
  workspaceId: string;
  entityId: string;
  entityType: string;
}

export interface IDiscoverReplySuccess {
  success: boolean;
  error?: never;
}

export interface IDiscoverReplyFailure {
  success?: never;
  error: SDFError;
}

export type IDiscoverReply = IDiscoverReplySuccess | IDiscoverReplyFailure;

export async function discover(
  request: IDiscoverRequest,
): Promise<IDiscoverReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IDiscoverReply = await sdf.post(
    "attributeDal/discover",
    request,
  );
  return reply;
}

export interface IImportImplementationRequest {
  workspaceId: string;
  implementationEntityId: string;
  entityId: string;
  applicationId: string;
  withConcept?: true;
}

export interface IImportImplementationReplySuccess {
  success: boolean;
  error?: never;
}

export interface IImportImplementationReplyFailure {
  success?: never;
  error: SDFError;
}

export type IImportImplementationReply =
  | IImportImplementationReplySuccess
  | IImportImplementationReplyFailure;

export async function importImplementation(
  request: IImportImplementationRequest,
): Promise<IImportImplementationReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IImportImplementationReply = await sdf.post(
    "attributeDal/importImplementation",
    request,
  );
  return reply;
}

export interface IImportConceptRequest {
  workspaceId: string;
  implementationEntityId: string;
  applicationId: string;
}

export interface IImportConceptReplySuccess {
  success: boolean;
  error?: never;
}

export interface IImportConceptReplyFailure {
  success?: never;
  error: SDFError;
}

export type IImportConceptReply =
  | IImportConceptReplySuccess
  | IImportConceptReplyFailure;

export async function importConcept(
  request: IImportConceptRequest,
): Promise<IImportConceptReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IImportConceptReply = await sdf.post(
    "attributeDal/importConcept",
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
  ``;
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IGetEntityReply = await sdf.get(
    "attributeDal/getEntity",
    request,
  );
  return reply;
}

export interface Connections {
  inbound: Connection[];
  outbound: Connection[];
}

export interface IGetConnectionsRequest {
  workspaceId: string;
  entityId: string;
  changeSetId?: string;
  editSessionId?: string;
}

export interface IGetConnectionsReplySuccess {
  connections: Connections;
  error?: never;
}

export interface IGetConnectionsReplyFailure {
  connections?: never;
  error: SDFError;
}

export type IGetConnectionsReply =
  | IGetConnectionsReplySuccess
  | IGetConnectionsReplyFailure;

export async function getConnections(
  request: IGetConnectionsRequest,
): Promise<IGetConnectionsReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IGetConnectionsReply = await sdf.get(
    "attributeDal/getConnections",
    request,
  );
  return reply;
}

export interface IDeleteConnectionRequest {
  workspaceId: string;
  changeSetId?: string;
  editSessionId?: string;
  edgeId: string;
}

export interface IDeleteConnectionReplySuccess {
  deleted: Boolean;
  error?: never;
}

export interface IDeleteConnectionReplyFailure {
  deleted?: never;
  error: SDFError;
}

export type IDeleteConnectionReply =
  | IDeleteConnectionReplySuccess
  | IDeleteConnectionReplyFailure;

export async function deleteConnection(
  request: IDeleteConnectionRequest,
): Promise<IDeleteConnectionReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IDeleteConnectionReply = await sdf.post(
    "attributeDal/deleteConnection",
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

export interface ICheckQualificationsRequest {
  workspaceId: string;
  entityId: string;
  changeSetId: string;
  editSessionId: string;
  systemId?: string;
}

export interface ICheckQualificationsReplySuccess {
  success: true;
  error?: never;
}

export interface ICheckQualificationsReplyFailure {
  error: SDFError;
}

export type ICheckQualificationsReply =
  | ICheckQualificationsReplySuccess
  | ICheckQualificationsReplyFailure;

export async function checkQualifications(
  request: ICheckQualificationsRequest,
): Promise<ICheckQualificationsReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: ICheckQualificationsReply = await sdf.post(
    "attributeDal/checkQualifications",
    request,
  );
  return reply;
}

export const AttributeDal = {
  getEntityList,
  getEntity,
  getConnections,
  deleteConnection,
  updateEntity,
  getInputLabels,
  checkQualifications,
  getDiscoveryList,
  discover,
  getImplementationsList,
  importImplementation,
  importConcept,
};
