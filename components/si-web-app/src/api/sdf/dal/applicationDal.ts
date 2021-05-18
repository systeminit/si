import { Entity } from "@/api/sdf/model/entity";
import { Resource } from "@/api/sdf/model/resource";
import { System } from "@/api/sdf/model/system";
import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";
import _ from "lodash";
import { WorkflowRun } from "../model/workflow";

export interface IApplicationCreateRequest {
  applicationName: string;
  workspaceId: string;
}

export interface IApplicationCreateReplySuccess {
  application: Entity;
  servicesWithResources: IServiceWithResources[];
  systems: System[];
  changeSetCounts: IChangeSetCounts;
  error?: never;
}

export interface IApplicationCreateReplyFailure {
  application?: never;
  servicesWithResources?: never;
  systems?: never;
  changeSetCounts?: never;
  error: SDFError;
}

export type IApplicationCreateReply =
  | IApplicationCreateReplySuccess
  | IApplicationCreateReplyFailure;

export interface IServiceWithResources {
  service: Entity;
  resources: Resource[];
}

export interface IChangeSetCounts {
  open: number;
  closed: number;
}

export async function createApplication(
  request: IApplicationCreateRequest,
): Promise<IApplicationCreateReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IApplicationCreateReply = await sdf.post(
    "applicationDal/createApplication",
    request,
  );

  if (!reply.error) {
    reply.application = Entity.fromJson(reply.application);
    reply.systems = _.map(reply.systems, isystem => {
      return System.upgrade(isystem);
    });
    reply.servicesWithResources = _.map(reply.servicesWithResources, iswr => {
      return {
        service: Entity.fromJson(iswr.service),
        resources: _.map(iswr.resources, r => {
          return Resource.upgrade(r);
        }),
      };
    });
  }
  return reply;
}

export interface IApplicationListRequest {
  workspaceId: string;
}

export interface IApplicationListReplySuccess {
  list: Omit<IApplicationCreateReplySuccess, "error">[];
  error?: never;
}

export interface IApplicationListReplyFailure {
  list?: never;
  error: SDFError;
}

export type IApplicationListReply =
  | IApplicationListReplySuccess
  | IApplicationListReplyFailure;

export async function listApplications(
  request: IApplicationListRequest,
): Promise<IApplicationListReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const listReply: IApplicationListReply = await sdf.get(
    "applicationDal/listApplications",
    request,
  );

  if (!listReply.error) {
    for (const reply of listReply.list) {
      reply.application = Entity.fromJson(reply.application);
      reply.systems = _.map(reply.systems, isystem => {
        return System.upgrade(isystem);
      });
      reply.servicesWithResources = _.map(reply.servicesWithResources, iswr => {
        return {
          service: Entity.fromJson(iswr.service),
          resources: _.map(iswr.resources, r => {
            return Resource.upgrade(r);
          }),
        };
      });
    }
  }
  return listReply;
}

export interface IDeployServicesRequest {
  workspaceId: string;
  systemId: string;
  applicationId: string;
}

export interface IDeployServicesReplySuccess {
  workflowRun: WorkflowRun;
  error?: never;
}

export interface IDeployServicesReplyFailure {
  workflowRun?: never;
  error: SDFError;
}

export type IDeployServicesReply =
  | IDeployServicesReplySuccess
  | IDeployServicesReplyFailure;

export async function deployServices(
  request: IDeployServicesRequest,
): Promise<IDeployServicesReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IDeployServicesReply = await sdf.post(
    "applicationDal/deployServices",
    request,
  );
  return reply;
}

export const ApplicationDal = {
  createApplication,
  listApplications,
  deployServices,
};
