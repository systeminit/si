import * as express from "express";
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

interface IntelligenceRequest {
  objectType: string;
  entity: Entity;
}

interface IntelligenceResponse {
  entity: Entity;
}

// TODO: Think through things like expression properties, setProperties, and the actual final properties.
//
// expressionProperties
// manualProperties
// inferredProperties

export function intelligence(
  req: express.Request,
  res: express.Response,
): void {
  console.log("POST /intelligence resolver begins");
  const intelReq: IntelligenceRequest = req.body;
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
  const intelRes: IntelligenceResponse = {
    entity: intelReq.entity,
  };
  res.send(intelRes);
}
