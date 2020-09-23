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
    baseline: Record<string, any>; // eslint-disable-line
  };
  manualProperties: {
    baseline: Record<string, any>; // eslint-disable-line
  };
  inferredProperties: {
    baseline: Record<string, any>; // eslint-disable-line
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

interface CalculatePropertiesRequest {
  objectType: string;
  entity: Entity;
}

interface CalculatePropertiesReply {
  entity: Entity;
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
  let registryObj;
  try {
    registryObj = registry.get(intelReq.objectType + "Entity") as EntityObject;
  } catch (err) {
    res.status(400);
    res.send({
      code: 400,
      message: `Cannot find registry object for ${intelReq.objectType}Entity`,
    });
    return;
  }
  registryObj.calculateProperties(intelReq.entity);
  const intelRes: CalculatePropertiesReply = {
    entity: intelReq.entity,
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
  path: string;
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
  res.send(opReply);
}
