import { ApiResponse } from "@/api/sdf";
import Bottle from "bottlejs";
import { User } from "@/api/sdf/dal/user";
import { BillingAccount } from "@/api/sdf/dal/billing_account";
import { user$ } from "@/observable/user";
import { billingAccount$ } from "@/observable/billing_account";

export interface LoginRequest {
  billingAccountName: string;
  userEmail: string;
  userPassword: string;
}

export interface LoginResponse {
  user: User;
  billingAccount: BillingAccount;
  jwt: string;
}

// NOTE: If you change this functions behavior, make sure you
// also change the cypress command for signup and login to match!
export async function login(
  request: LoginRequest,
): Promise<ApiResponse<LoginResponse>> {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;

  const response: LoginResponse = await sdf.post("session/login", request);
  sdf.token = response.jwt;
  user$.next(response.user);
  billingAccount$.next(response.billingAccount);
  return response;
}
