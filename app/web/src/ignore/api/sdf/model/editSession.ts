import { ISiStorable } from "@/api/sdf/model/siStorable";

export interface IEditSession {
  id: string;
  name: string;
  note: string;
  reverted: boolean;
  changeSetId: string;
  siStorable: ISiStorable;
}

export class EditSession implements IEditSession {
  id: IEditSession["id"];
  name: IEditSession["name"];
  note: IEditSession["note"];
  reverted: IEditSession["reverted"];
  changeSetId: IEditSession["changeSetId"];
  siStorable: IEditSession["siStorable"];

  constructor(args: IEditSession) {
    this.id = args.id;
    this.name = args.name;
    this.note = args.note;
    this.reverted = args.reverted;
    this.changeSetId = args.changeSetId;
    this.siStorable = args.siStorable;
  }
}
