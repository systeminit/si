import { StandardModel } from "@/api/sdf/dal/standard_model";

export enum SchemaKind {
  Concept = "concept",
  Implementation = "implementation",
  Concrete = "concrete",
}

export interface Schema extends StandardModel {
  name: string;
  kind: SchemaKind;
  ui_menu_name: string;
  ui_menu_category: string;
  ui_hidden: boolean;
}
