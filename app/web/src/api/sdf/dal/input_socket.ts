import { StandardModel } from "@/api/sdf/dal/standard_model";

export interface InputSocket extends StandardModel {
  name?: string;
  internalOnly: boolean;
  type_definition?: string;
  propId: number;
  schemaId: number;
  schemaVariantId: number;
}
