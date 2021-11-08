export interface Tenancy {
  tenancy_universal: boolean;
  tenancy_billing_account_ids: Array<number>;
  tenancy_organization_ids: Array<number>;
  tenancy_workspace_ids: Array<number>;
}
