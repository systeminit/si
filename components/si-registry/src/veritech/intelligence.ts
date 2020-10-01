import * as express from "express";
import _ from "lodash";

import { registry } from "@/registry";
import { EntityObject } from "@/systemComponent";

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
  object: object;
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
