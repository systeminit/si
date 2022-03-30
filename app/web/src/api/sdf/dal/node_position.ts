import { Tenancy } from "@/api/sdf/dal/tenancy";
import { Visibility } from "@/api/sdf/dal/visibility";

export interface NodePosition {
  pk: number;
  id: number;
  schematic_kind: number;
  deployment_node_id?: number;
  root_node_id: number;
  system_id?: number;
  x: string;
  y: string;
  tenancy: Tenancy;
  timestamp: Date;
  visibility: Visibility;
}
