import * as express from "express";
import _ from "lodash";
import YAML from "yaml";

import { registry } from "../registry";
import { EntityObject } from "../systemComponent";

export type NodeObject = Entity | System;

export interface Entity {
  id: string;
  name: string;
  objectType: string;
  description: string;
  nodeId: string;
  expressionProperties: {
    __baseline: Record<string, any>; // eslint-disable-line
  };
  manualProperties: {
    __baseline: Record<string, any>; // eslint-disable-line
  };
  inferredProperties: {
    __baseline: Record<string, any>; // eslint-disable-line
  };
  properties: {
    __baseline: Record<string, any>; //eslint-disable-line
  };
  siStorable: {
    typeName: string;
    objectId: string;
    billingAccountId: string;
    organizationId: string;
    workspaceId: string;
    tenantIds: string[];
    createdByUserId: string;
    updateClock: {
      epoch: string;
      updateCount: string;
    };
    deleted: boolean;
  };
}

export interface System {
  id: string;
  name: string;
  description: string;
  nodeId: string;
  head: boolean;
  siStorable: Entity["siStorable"];
}

export enum ResourceHealth {
  Ok = "ok",
  Warning = "warning",
  Error = "error",
  Unknown = "unknown",
}

export enum ResourceStatus {
  Pending = "pending",
  InProgress = "inProgress",
  Created = "created",
  Failed = "failed",
  Deleted = "deleted",
}

export interface Resource {
  id: string;
  unixTimestamp: number;
  timestamp: string;
  state: any;
  status: ResourceStatus;
  health: ResourceHealth;
  systemId: string;
  nodeId: string;
  entityId: string;
  siStorable: Entity["siStorable"];
}

export interface ResourceUpdate {
  state: any;
  status: ResourceStatus;
  health: ResourceHealth;
}

interface CalculatePropertiesRequest {
  objectType: string;
  entity: Entity;
}

interface CalculatePropertiesReply {
  entity: Entity;
}

export interface CalculatePropertiesResult {
  properties: {
    __baseline: Record<string, any>; // eslint-disable-line
    [key: string]: Record<string, any>; // eslint-disable-line
  };
}

// TODO: Think through things like expression properties, setProperties, and the actual final properties.
//
// expressionProperties
// manualProperties
// inferredProperties

export function calculateProperties(
  req: express.Request,
  res: express.Response,
): void {
  console.log("POST /calculateProperties resolver begins");
  const intelReq: CalculatePropertiesRequest = req.body;
  const entity = intelReq.entity;
  let registryObj;
  try {
    registryObj = registry.get(intelReq.objectType) as EntityObject;
  } catch (err) {
    res.status(400);
    res.send({
      code: 400,
      message: `Cannot find registry object for ${intelReq.objectType}`,
    });
    return;
  }
  const result: CalculatePropertiesResult = registryObj.calculateProperties(
    entity,
  );
  entity.properties = result.properties;
  console.dir(entity, { depth: Infinity });
  //console.log("sending back the entity", { entity });
  const intelRes: CalculatePropertiesReply = {
    entity,
  };
  res.send(intelRes);
}

enum Operation {
  Set = "set",
  Unset = "unset",
}

interface ApplyOpRequest {
  operation: Operation;
  toId: string;
  path: string[];
  // eslint-disable-next-line
  value?: any;
  object: NodeObject;
}

interface ApplyOpReply {
  object: object;
}

export function applyOp(req: express.Request, res: express.Response): void {
  console.log("POST /applyOp resolver begins");
  const opRequest: ApplyOpRequest = req.body;
  console.dir(opRequest, { depth: Infinity });
  const object = opRequest.object;
  if (opRequest.operation == Operation.Set) {
    if (opRequest.value) {
      if (opRequest.path.includes("kubernetesObjectYaml")) {
        const jsYaml = YAML.parse(opRequest.value);
        _.set(
          object,
          ["manualProperties", "__baseline", "kubernetesObject"],
          jsYaml,
        );
      }
      _.set(object, opRequest.path, opRequest.value);
    } else {
      res.status(400).send({
        error: "operation was set, but no value was passed!",
      });
      return;
    }
  } else if (opRequest.operation == Operation.Unset) {
    _.unset(object, opRequest.path);
  }

  const opReply: ApplyOpReply = {
    object,
  };
  console.log("sending applyOp reply");
  console.dir(opReply, { depth: Infinity });
  res.send(opReply);
}

interface CalculateConfiguresRequest {
  entity: Entity;
  configures: Entity[];
  systems: System[];
}

export interface CalculateConfiguresReply {
  keep?: {
    id: string;
    systems: string[];
  }[];
  create?: {
    objectType: string;
    name?: string;
    systems: string[];
  }[];
}

export function calculateConfigures(
  req: express.Request,
  res: express.Response,
): void {
  console.log("POST /calculateConfigures resolver begins");
  const intelReq: CalculateConfiguresRequest = req.body;
  const entity = intelReq.entity;
  const configures = intelReq.configures;
  const systems = intelReq.systems;
  const objectType = intelReq.entity.objectType;
  let registryObj;
  try {
    registryObj = registry.get(objectType) as EntityObject;
  } catch (err) {
    res.status(400);
    res.send({
      code: 400,
      message: `Cannot find registry object for ${objectType}`,
    });
    return;
  }
  const response: CalculateConfiguresReply = registryObj.calculateConfigures(
    entity,
    configures,
    systems,
  );
  console.dir(response, { depth: Infinity });
  console.log("sending response", { response });
  res.send(response);
}

export interface ActionRequest {
  toId: string;
  action: string;
  systemId: string;
  hypothetical: boolean;
  entities: {
    successors: Entity[];
    predecessors: Entity[];
  };
  resources: {
    successors: Resource[];
    predecessors: Resource[];
  };
  entity: Entity;
}

export interface ActionReply {
  resource: ResourceUpdate;
  actions: {
    action: string;
    entityId: string;
  }[];
}

export function action(req: express.Request, res: express.Response): void {
  console.log("POST /action resolver begins");
  const request: ActionRequest = req.body;
  console.dir(request, { depth: Infinity });
  let registryObj;
  try {
    registryObj = registry.get(request.entity.objectType) as EntityObject;
  } catch (err) {
    res.status(400);
    res.send({
      code: 400,
      message: `Cannot find registry object for ${request.entity.objectType}`,
    });
    return;
  }

  registryObj
    .action(request)
    .then(reply => {
      console.log("action reply");
      console.dir(reply, { depth: Infinity });
      res.send(reply);
    })
    .catch(err => {
      res.status(400);
      res.send({
        code: 400,
        messsage: `Cannot execute action for ${request.entity.objectType}: ${err}`,
      });
    });
}

export interface SyncResourceRequest {
  systemId: string;
  node: Node;
  entity: Entity;
  resource: Resource;
}

export interface SyncResourceReply {
  resource: ResourceUpdate;
}

export function syncResource(
  req: express.Request,
  res: express.Response,
): void {
  console.log("POST /syncResource resolver begins");
  const request: SyncResourceRequest = req.body;
  console.dir(request, { depth: Infinity });
  let registryObj;
  try {
    registryObj = registry.get(request.entity.objectType) as EntityObject;
  } catch (err) {
    res.status(400);
    res.send({
      code: 400,
      message: `Cannot find registry object for ${request.entity.objectType}`,
    });
    return;
  }

  registryObj
    .syncResource(request)
    .then(reply => {
      console.log("sync reply");
      console.dir(reply, { depth: Infinity });
      res.send(reply);
    })
    .catch(err => {
      res.status(400);
      res.send({
        code: 400,
        messsage: `Cannot execute sync for ${request.entity.objectType}: ${err}`,
      });
    });
}
