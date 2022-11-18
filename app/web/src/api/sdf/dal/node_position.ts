import { Tenancy } from "@/api/sdf/dal/tenancy";
import { Visibility } from "@/api/sdf/dal/visibility";

export interface NodePosition {
  pk: number;
  id: number;
  diagram_kind: number;
  x: string;
  y: string;
  tenancy: Tenancy;
  timestamp: Date;
  visibility: Visibility;
}
