import { IUpdateClock } from "./updateClock";

export interface ISiStorable {
  typeName: string;
  objectId: string;
  billingAccountId: string;
  organizationId: string;
  workspaceId: string;
  tenantIds: string[];
  createdByUserId?: string;
  updateClock: IUpdateClock;
  deleted: boolean;
}

export interface ISimpleStorable {
  typeName: String;
  objectId: String;
  billingAccountId: String;
  tenantIds: String[];
  deleted: boolean;
}

export interface IMinimalStorable {
  typeName: String;
  objectId: String;
  deleted: boolean;
}
