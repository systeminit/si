import { Tenancy } from "@/api/sdf/dal/tenancy";
import { Visibility } from "@/api/sdf/dal/visibility";

export interface NodePosition {
  pk: number;
  id: number;
  schematicKind: number;
  rootNodeId: number;
  systemId?: number;
  x: string;
  y: string,
  tenancy: Tenancy,
  timestamp: Date,
  visibility: Visibility,
}
