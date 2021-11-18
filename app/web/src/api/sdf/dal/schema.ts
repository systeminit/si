import { StandardModel } from "@/api/sdf/dal/standard_model";

export enum SchemaKind {
  Concept = "concept",
  Implementation = "implementation",
  Concrete = "concrete",
}

export interface Schema extends StandardModel {
  name: string;
  kind: SchemaKind;
  ui_menu_name: String;
  ui_menu_category: String;
  ui_hidden: boolean;
}
