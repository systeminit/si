import { Visibility } from "@/api/sdf/dal/visibility";
import { Tenancy } from "@/api/sdf/dal/tenancy";

export interface StandardModel extends Visibility, Tenancy {
  pk: number;
  id: number;
}

export interface StandardModelNoVisibility extends Tenancy {
  pk: number;
  id: number;
}
