import { BillingAccount } from "@/api/sdf/model/billingAccount";
import { User } from "@/api/sdf/model/user";
import { SDFError, SDF } from "@/api/sdf";
import * as jwtLib from "jsonwebtoken";
import Bottle from "bottlejs";

export interface ISessionDalLoginRequest {
  billingAccountName: string;
  userEmail: string;
  userPassword: string;
}

export interface ISessionDalLoginReplySuccess {
  user: User;
  billingAccount: BillingAccount;
  jwt: string;
  error?: never;
}

export interface ISessionDalLoginReplyFailure {
  error: SDFError;
  billingAccount?: never;
  user?: never;
  jwt?: never;
}

export type ISessionDalLoginReply =
  | ISessionDalLoginReplySuccess
  | ISessionDalLoginReplyFailure;

export interface ISessionDalIsAuthenticatedRequest {
  billingAccount: BillingAccount | null;
  user: User | null;
}

export interface ISessionDalIsAuthenticatedReplySuccess {
  user: User;
  billingAccount: BillingAccount;
  error?: never;
  logout?: never;
  login?: never;
}

export interface ISessionDalIsAuthenticatedReplyFailure {
  user?: never;
  billingAccount?: never;
  error: SDFError;
  logout?: never;
  login?: never;
}

export interface ISessionDalIsAuthenticatedReplyLogout {
  user?: never;
  billingAccount?: never;
  error?: never;
  logout: true;
  login?: never;
}

export interface ISessionDalIsAuthenticatedReplyLogin {
  user?: never;
  billingAccount?: never;
  error?: never;
  logout?: never;
  login: true;
}

export type ISessionDalIsAuthenticatedReply =
  | ISessionDalIsAuthenticatedReplyLogout
  | ISessionDalIsAuthenticatedReplyLogin
  | ISessionDalIsAuthenticatedReplySuccess
  | ISessionDalIsAuthenticatedReplyFailure;

export interface ISessionDalRestoreAuthenticationReplySuccess {
  user: User;
  billingAccount: BillingAccount;
  error?: never;
}

export interface ISessionDalRestoreAuthenticationReplyFailure {
  error: SDFError;
  billingAccount?: never;
  user?: never;
}

export type ISessionDalRestoreAuthenticationReply =
  | ISessionDalRestoreAuthenticationReplySuccess
  | ISessionDalRestoreAuthenticationReplyFailure;

export class SessionDal {
  static async login(
    request: ISessionDalLoginRequest,
  ): Promise<ISessionDalLoginReply> {
    let bottle = Bottle.pop("default");
    let sdf: SDF = bottle.container.SDF;

    const loginReply: ISessionDalLoginReply = await sdf.post(
      "sessionDal/login",
      request,
    );
    if (!loginReply.error) {
      loginReply.billingAccount = new BillingAccount(loginReply.billingAccount);
      loginReply.user = new User(loginReply.user);
      sdf.token = loginReply.jwt;
    }
    return loginReply;
  }

  static async logout() {
    let bottle = Bottle.pop("default");
    let sdf: SDF = bottle.container.SDF;

    sdf.token = undefined;
    if (sdf.update) {
      sdf.update.socket.close();
    }
  }

  static async isAuthenticated(
    request: ISessionDalIsAuthenticatedRequest,
  ): Promise<ISessionDalIsAuthenticatedReply> {
    let bottle = Bottle.pop("default");
    let sdf: SDF = bottle.container.SDF;
    const token = sdf.token;
    if (token) {
      let currentTime = Math.floor(Date.now() / 1000);
      let decodedToken = jwtLib.decode(token, {
        complete: true,
      }) as any;
      if (decodedToken && currentTime >= decodedToken["payload"]["exp"]) {
        return { logout: true };
      }
      if (!request.user || !request.billingAccount) {
        let restoreReply: ISessionDalRestoreAuthenticationReply = await sdf.get(
          "sessionDal/restoreAuthentication",
        );
        return restoreReply;
      } else {
        return { user: request.user, billingAccount: request.billingAccount };
      }
    } else {
      return { login: true };
    }
  }
}
