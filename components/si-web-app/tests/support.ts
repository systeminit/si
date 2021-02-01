import {
  uniqueNamesGenerator,
  adjectives,
  colors,
  animals,
} from "unique-names-generator";
import { SignupDal, ISignupDalReply } from "@/api/sdf/dal/signupDal";
import { SessionDal } from "@/api/sdf/dal/sessionDal";
import { User } from "@/api/sdf/model/user";
import { BillingAccount } from "@/api/sdf/model/billingAccount";

export function createFakeName(): string {
  const randomName: string = uniqueNamesGenerator({
    dictionaries: [adjectives, colors, animals],
  });
  return randomName;
}

export interface INewBillingAccount {
  billingAccount: BillingAccount;
  user: User;
}

export async function createBillingAccountAndLogin(): Promise<
  INewBillingAccount
> {
  const billingAccountName = createFakeName();

  const reply = await SignupDal.createBillingAccount({
    billingAccountName,
    billingAccountDescription: "acme",
    userName: "a",
    userEmail: "a",
    userPassword: "a",
  });
  if (reply.error) {
    throw new Error(reply.error.message);
  }

  const loginReply = await SessionDal.login({
    billingAccountName,
    userEmail: "a",
    userPassword: "a",
  });
  if (loginReply.error) {
    throw new Error(loginReply.error.message);
  }

  return { billingAccount: loginReply.billingAccount, user: loginReply.user };
}
