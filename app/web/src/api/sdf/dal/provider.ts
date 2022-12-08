import { StandardModel } from "@/api/sdf/dal/standard_model";

export interface InternalProvider extends StandardModel {
  propId: string;
  schemaId: string;
  schemaVariantId: string;
  name?: string;
  inbound_type_definition?: string;
  outbound_type_definition?: string;
}

export interface ExternalProvider extends StandardModel {
  propId: string;
  schemaId: string;
  schemaVariantId: string;
  name?: string;
  type_definition?: string;
  attribute_prototype_id: string;
}
