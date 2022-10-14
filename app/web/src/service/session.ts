import { sdf } from "@/api/sdf";
import { BillingAccount } from "@/api/sdf/dal/billing_account";
import { User } from "@/api/sdf/dal/user";
import { workspace$ } from "@/observable/workspace";

export interface LoginResponse {
  user: User;
  billingAccount: BillingAccount;
  jwt: string;
}

function setAuth(response: LoginResponse) {
  sdf.token = response.jwt;
  workspace$.next(null);
}

async function logout() {
  sdf.token = undefined;
  // not sure about this... but will leave for now
  sessionStorage.clear();
}

export const SessionService = {
  setAuth,
  logout,
};
