import { StandardModelNoVisibility } from "@/api/sdf/dal/standard_model";

export enum EditSessionStatus {
  Open = "Open",
  Canceled = "Canceled",
  Saved = "Saved",
}

export interface EditSession extends StandardModelNoVisibility {
  name: string;
  note?: string;
  status: EditSessionStatus;
}
