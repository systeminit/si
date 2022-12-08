import { Visibility } from "@/api/sdf/dal/visibility";
import { Tenancy } from "@/api/sdf/dal/tenancy";

export interface StandardModel extends Visibility, Tenancy {
  pk: string;
  id: string;
}

export interface StandardModelNoVisibility extends Tenancy {
  pk: string;
  id: string;
}
