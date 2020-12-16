import * as express from "express";
import _ from "lodash";
import YAML from "yaml";
import WebSocket from "ws";

import { registry } from "../registry";
import { EntityObject } from "../systemComponent";
import { Event, EventLogLevel } from "./eventLog";

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
    [key: string]: Record<string, any>; // eslint-disable-line
  };
  inferredProperties: {
    __baseline: Record<string, any>; // eslint-disable-line
    [key: string]: Record<string, any>; // eslint-disable-line
  };
  properties: {
    __baseline: Record<string, any>; //eslint-disable-line
    [key: string]: Record<string, any>; //eslint-disable-line
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

export interface CalculatePropertiesRequest {
  objectType: string;
  entity: Entity;
  resources: Resource[];
  predecessors: {
    entity: Entity;
    resources: Resource[];
  }[];
}

export interface CalculatePropertiesReply {
  entity: Entity;
}

export interface CalculatePropertiesResult {
  inferredProperties: {
    __baseline: Record<string, any>; // eslint-disable-line
    [key: string]: Record<string, any>; // eslint-disable-line
  };
}

export interface CalculatePropertiesFullResult {
  inferredProperties: CalculatePropertiesResult["inferredProperties"];
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
  console.log("POST /calculateProperties BEGIN");
  const intelReq: CalculatePropertiesRequest = req.body;
  console.dir(intelReq, { depth: Infinity });
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
  const result: CalculatePropertiesFullResult = registryObj.calculateProperties(
    intelReq,
  );
  entity.properties = result.properties;
  entity.inferredProperties = result.inferredProperties;
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
  console.log("POST /applyOp BEGIN");
  const opRequest: ApplyOpRequest = req.body;
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
  console.log("POST /calculateConfigures BEGIN");
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
  res.send(response);
}

export interface ActionRequest {
  action: string;
  systemId: string;
  node: Node;
  entity: Entity;
  resource: Resource;
  hypothetical: boolean;
  predecessors: {
    entity: Entity;
    resource: Resource;
  }[];
  successors: {
    entity: Entity;
    resource: Resource;
  }[];
}

export interface ActionReply {
  resource: ResourceUpdate;
  actions: {
    action: string;
    entityId: string;
  }[];
}

export function action(ws: WebSocket, req: string): void {
  console.log("POST /action BEGIN");
  const request: ActionRequest = JSON.parse(req);
  let registryObj;
  try {
    registryObj = registry.get(request.entity.objectType) as EntityObject;
  } catch (err) {
    ws.close(
      4004,
      `cannot find registry object for ${request.entity.objectType}`,
    );
    return;
  }

  const event = new Event(ws);

  registryObj
    .action(request, event)
    .then(reply => {
      console.log("action reply");
      console.dir(reply, { depth: Infinity });
      ws.send(JSON.stringify({ reply: reply }));
      ws.close(1000, "finished");
    })
    .catch(err => {
      ws.close(
        4004,
        `Cannot execute action for ${request.entity.objectType}: ${err}`,
      );
    });
}

export interface SyncResourceRequest {
  systemId: string;
  node: Node;
  entity: Entity;
  resource: Resource;
  predecessors: {
    entity: Entity;
    resource: Resource;
  }[];
}

export interface SyncResourceReply {
  resource: ResourceUpdate;
}

export function syncResource(ws: WebSocket, req: string): void {
  console.log("/ws/syncResource resolver begins");
  const request = JSON.parse(req);
  console.dir(request, { depth: Infinity });
  let registryObj;
  try {
    registryObj = registry.get(request.entity.objectType) as EntityObject;
  } catch (err) {
    ws.close(
      4004,
      `cannot find registry object for ${request.entity.objectType}`,
    );
    return;
  }

  const event = new Event(ws);

  registryObj
    .syncResource(request, event)
    .then(reply => {
      console.log("sync reply");
      console.dir(reply, { depth: Infinity });
      ws.send(JSON.stringify({ reply: reply }));
      ws.close(1000, "finished");
    })
    .catch(err => {
      console.log(`Failed to execute sync ${err}`, err);
      event.log(EventLogLevel.Fatal, `resource sync failed: ${err}`, { err });
      const r = {
        resource: {
          state: { ...request.resource.state, errorMsg: `${err}` },
          health: ResourceHealth.Error,
          status: ResourceStatus.Failed,
        },
      };
      ws.send(JSON.stringify({ reply: r }));
      ws.close(1000, "finished");
    });
}

export function findEntityByType(
  graph: { entity: Entity }[],
  objectType: string,
): Entity | undefined {
  const result = _.find(graph, ["entity.objectType", objectType]);
  if (result) {
    return result.entity;
  } else {
    return undefined;
  }
}
