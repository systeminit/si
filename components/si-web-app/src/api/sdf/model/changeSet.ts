import { ISiStorable, ISimpleStorable } from "@/api/sdf/model/siStorable";

export enum ChangeSetStatus {
  Open = "open",
  Closed = "closed",
  Abandoned = "abandoned",
  Executing = "executing",
  Failed = "failed",
}

export interface IChangeSet {
  id: string;
  name: string;
  note: string;
  status: ChangeSetStatus;
  siStorable: ISiStorable;
}

export class ChangeSet implements IChangeSet {
  id: IChangeSet["id"];
  name: IChangeSet["name"];
  note: IChangeSet["note"];
  status: IChangeSet["status"];
  siStorable: IChangeSet["siStorable"];

  constructor(args: IChangeSet) {
    this.id = args.id;
    this.name = args.name;
    this.note = args.note;
    this.status = args.status;
    this.siStorable = args.siStorable;
  }
}

export interface IChangeSetParticipant {
  id: string;
  changeSetId: string;
  objectId: string;
  siStorable: ISimpleStorable;
}

export class ChangeSetParticipant implements IChangeSetParticipant {
  id: IChangeSetParticipant["id"];
  changeSetId: IChangeSetParticipant["changeSetId"];
  objectId: IChangeSetParticipant["objectId"];
  siStorable: IChangeSetParticipant["siStorable"];

  constructor(args: IChangeSetParticipant) {
    this.id = args.id;
    this.changeSetId = args.changeSetId;
    this.objectId = args.objectId;
    this.siStorable = args.siStorable;
  }
}
