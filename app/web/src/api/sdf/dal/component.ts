import { StandardModel } from "@/api/sdf/dal/standard_model";
import { SchematicKind } from "@/api/sdf/dal/schematic";

export interface Component extends StandardModel {
  name: string;
}

export interface ComponentIdentification {
  componentId: number;
  schemaVariantId: number;
  schemaVariantName: string;
  schemaId: number;
  schemaName: string;
  schematicKind: SchematicKind;
}
