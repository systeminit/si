import { ChangeSet } from "@/api/sdf/model/changeSet";
import { EditSession } from "@/api/sdf/model/editSession";
import { SDFError } from "@/api/sdf";
import { ILabelList } from "@/api/sdf/dal";
import Bottle from "bottlejs";
import _ from "lodash";

export interface IGetApplicationContextRequest {
  workspaceId: string;
  applicationId: string;
}

export interface IGetApplicationContextReplySuccess {
  applicationName: string;
  systemsList: ILabelList;
  openChangeSetsList: ILabelList;
  error?: never;
}

export interface IGetApplicationContextReplyFailure {
  applicationName?: never;
  systemsList?: never;
  openChangeSetsList?: never;
  error: SDFError;
}

export type IGetApplicationContextReply =
  | IGetApplicationContextReplySuccess
  | IGetApplicationContextReplyFailure;

export interface ICreateChangeSetAndEditSessionRequest {
  workspaceId: string;
  changeSetName: string;
}

export interface ICreateChangeSetAndEditSessionReplySuccess {
  changeSet: ChangeSet;
  editSession: EditSession;
  error?: never;
}

export interface ICreateChangeSetAndEditSessionReplyFailure {
  changeSet?: never;
  editSession?: never;
  error: SDFError;
}

export type ICreateChangeSetAndEditSessionReply =
  | ICreateChangeSetAndEditSessionReplySuccess
  | ICreateChangeSetAndEditSessionReplyFailure;

export interface IGetChangeSetAndEditSessionRequest {
  changeSetId: string;
  editSessionId: string;
}

export interface IGetChangeSetAndEditSessionReplySuccess {
  changeSet: ChangeSet;
  editSession: EditSession;
  error?: never;
}

export interface IGetChangeSetAndEditSessionReplyFailure {
  changeSet?: never;
  editSession?: never;
  error: SDFError;
}

export type IGetChangeSetAndEditSessionReply =
  | IGetChangeSetAndEditSessionReplySuccess
  | IGetChangeSetAndEditSessionReplyFailure;

export interface ICreateEditSessionRequest {
  changeSetId: string;
  workspaceId: string;
}

export interface ICreateEditSessionReplySuccess {
  editSession: EditSession;
  error?: never;
}

export interface ICreateEditSessionReplyFailure {
  editSession?: never;
  error: SDFError;
}

export type ICreateEditSessionReply =
  | ICreateEditSessionReplySuccess
  | ICreateEditSessionReplyFailure;

export interface ICancelEditSessionRequest {
  editSessionId: string;
  workspaceId: string;
}

export interface ICancelEditSessionReplySuccess {
  editSession: EditSession;
  error?: never;
}

export interface ICancelEditSessionReplyFailure {
  editSession?: never;
  error: SDFError;
}

export type ICancelEditSessionReply =
  | ICancelEditSessionReplySuccess
  | ICancelEditSessionReplyFailure;

export interface ICreateEditSessionAndGetChangeSetRequest {
  changeSetId: string;
}

export interface ICreateEditSessionAndGetChangeSetReplySuccess {
  changeSet: ChangeSet;
  editSession: EditSession;
  error?: never;
}

export interface ICreateEditSessionAndGetChangeSetReplyFailure {
  changeSet?: never;
  editSession?: never;
  error: SDFError;
}

export type ICreateEditSessionAndGetChangeSetReply =
  | ICreateEditSessionAndGetChangeSetReplySuccess
  | ICreateEditSessionAndGetChangeSetReplyFailure;

export class ApplicationContextDal {
  static async getApplicationContext(
    request: IGetApplicationContextRequest,
  ): Promise<IGetApplicationContextReply> {
    let bottle = Bottle.pop("default");
    let sdf = bottle.container.SDF;

    const listReply: IGetApplicationContextReply = await sdf.get(
      "applicationContextDal/getApplicationContext",
      request,
    );

    return listReply;
  }

  static async createChangeSetAndEditSession(
    request: ICreateChangeSetAndEditSessionRequest,
  ): Promise<ICreateChangeSetAndEditSessionReply> {
    let bottle = Bottle.pop("default");
    let sdf = bottle.container.SDF;

    const reply: ICreateChangeSetAndEditSessionReply = await sdf.post(
      "applicationContextDal/createChangeSetAndEditSession",
      request,
    );

    return reply;
  }

  static async getChangeSetAndEditSession(
    request: IGetChangeSetAndEditSessionRequest,
  ): Promise<IGetChangeSetAndEditSessionReply> {
    let bottle = Bottle.pop("default");
    let sdf = bottle.container.SDF;

    const reply: IGetChangeSetAndEditSessionReply = await sdf.get(
      "applicationContextDal/getChangeSetAndEditSession",
      request,
    );

    return reply;
  }

  static async createEditSessionAndGetChangeSet(
    request: ICreateEditSessionAndGetChangeSetRequest,
  ): Promise<ICreateEditSessionAndGetChangeSetReply> {
    let bottle = Bottle.pop("default");
    let sdf = bottle.container.SDF;

    const reply: ICreateEditSessionAndGetChangeSetReply = await sdf.post(
      "applicationContextDal/createEditSessionAndGetChangeSet",
      request,
    );

    return reply;
  }

  static async createEditSession(
    request: ICreateEditSessionRequest,
  ): Promise<ICreateEditSessionReply> {
    let bottle = Bottle.pop("default");
    let sdf = bottle.container.SDF;

    const reply: ICreateEditSessionReply = await sdf.post(
      "applicationContextDal/createEditSession",
      request,
    );

    return reply;
  }

  static async cancelEditSession(
    request: ICancelEditSessionRequest,
  ): Promise<ICancelEditSessionReply> {
    let bottle = Bottle.pop("default");
    let sdf = bottle.container.SDF;

    const reply: ICancelEditSessionReply = await sdf.post(
      "applicationContextDal/cancelEditSession",
      request,
    );

    return reply;
  }
}
