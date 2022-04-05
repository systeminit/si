import { StandardModel } from "@/api/sdf/dal/standard_model";

export interface Component extends StandardModel {
  name: string;
}

export interface ComponentWithSchemaAndVariant {
  componentId: number;
  schemaVariantId: number;
  schemaId: number;
}
