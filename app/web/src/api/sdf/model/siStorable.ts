//import { IUpdateClock } from "./updateClock";

export interface ISiStorable {
  typeName: string;
  objectId: string;
  billingAccountId: string;
  organizationId: string;
  workspaceId: string;
  tenantIds: string[];
  createdByUserId?: string;
  deleted: boolean;
}

export interface ISimpleStorable {
  typeName: string;
  objectId: string;
  billingAccountId: string;
  tenantIds: string[];
  deleted: boolean;
}

export interface IMinimalStorable {
  typeName: string;
  objectId: string;
  deleted: boolean;
}
