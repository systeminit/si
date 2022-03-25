import { ApiResponse, SDF } from "@/api/sdf";
import Bottle from "bottlejs";
import { User } from "@/api/sdf/dal/user";
import { BillingAccount } from "@/api/sdf/dal/billing_account";
import { user$ } from "@/observable/user";
import { billingAccount$ } from "@/observable/billing_account";
import { Observable, tap } from "rxjs";
import { workspace$ } from "@/observable/workspace";
import { system$ } from "@/observable/system";
import { organization$ } from "@/observable/organization";
import { switchToHead } from "@/service/change_set/switch_to_head";

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

export function login(
  request: LoginRequest,
): Observable<ApiResponse<LoginResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  return sdf.post<ApiResponse<LoginResponse>>("session/login", request).pipe(
    tap((response) => {
      if (!response.error) {
        sdf.token = response.jwt;
        switchToHead();
        workspace$.next(null);
        organization$.next(null);
        system$.next(null);
        user$.next(response.user);
        billingAccount$.next(response.billingAccount);
      }
    }),
  );
}
