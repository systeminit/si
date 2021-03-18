export interface SiStorable {
  typeName: string;
  objectId: string;
  billingAccountId: string;
  organizationId: string;
  workspaceId: string;
  tenantIds: string[];
  deleted: boolean;
}
