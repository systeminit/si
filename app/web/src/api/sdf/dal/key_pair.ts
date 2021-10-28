import { StandardModel } from "@/api/sdf/dal/standard_model";

export interface PublicKey extends StandardModel {
  name: string;
  public_key: string;
  created_lampot_clock: string;
}
