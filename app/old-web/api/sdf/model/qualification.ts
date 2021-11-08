import { ISiStorable } from "@/api/sdf/model/siStorable";
import { ISiChangeSet } from "@/api/sdf/model/siChangeSet";

export interface Qualification {
  id: string;
  entityId: string;
  name: string;
  qualified: boolean;
  output?: string;
  error?: string;
  siStorable: ISiStorable;
  siChangeSet: ISiChangeSet;
}

export interface QualificationStart {
  start: string;
  entityId: string;
  changeSetId: string;
  editSessionId: string;
  siStorable: ISiStorable;
}
