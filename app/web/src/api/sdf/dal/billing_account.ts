import { StandardModel } from "@/api/sdf/dal/standard_model";

export interface BillingAccount extends StandardModel {
  name: string;
  description?: string;
}
