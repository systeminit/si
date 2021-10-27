import {
  // IBillingAccount,
  BillingAccount,
} from "@/api/sdf/model/billingAccount";
import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";

export interface ISignupDalRequest {
  billingAccountName: string;
  billingAccountDescription: string;
  userName: string;
  userEmail: string;
  userPassword: string;
}

export interface ISignupDalReplyFailure {
  error: SDFError;
  billingAccount?: never;
}

export interface ISignupDalReplySuccess {
  billingAccount: BillingAccount;
  error?: never;
}

export type ISignupDalReply = ISignupDalReplySuccess | ISignupDalReplyFailure;

export class SignupDal {
  static async createBillingAccount(
    request: ISignupDalRequest,
  ): Promise<ISignupDalReply> {
    const bottle = Bottle.pop("default");
    const sdf = bottle.container.SDF;

    const billingAccountReply: ISignupDalReply = await sdf.post(
      "signupDal/createBillingAccount",
      request,
    );

    if (!billingAccountReply.error) {
      billingAccountReply.billingAccount = BillingAccount.upgrade(
        billingAccountReply.billingAccount,
      );
    }
    return billingAccountReply;
  }
}
