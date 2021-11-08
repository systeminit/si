import { StandardModel } from "@/api/sdf/dal/standard_model";

export interface Capability extends StandardModel {
  subject: string;
  action: string;
}
