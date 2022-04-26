import { firstValueFrom } from "rxjs";
import { user$ } from "@/observable/user";
import { billingAccount$ } from "@/observable/billing_account";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { User } from "@/api/sdf/dal/user";
import { BillingAccount } from "@/api/sdf/dal/billing_account";

interface RestoreAuthenticationResponse {
  user: User;
  billing_account: BillingAccount;
}

export async function isAuthenticated(): Promise<ApiResponse<boolean>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const token = sdf.token;
  if (token) {
    if (Date.now() >= JSON.parse(atob(token.split(".")[1])).exp * 1000) {
      return false;
    }
    const user = await firstValueFrom(user$);
    const billingAccount = await firstValueFrom(billingAccount$);
    if (!user && !billingAccount) {
      const result: ApiResponse<RestoreAuthenticationResponse> =
        await firstValueFrom(sdf.get("session/restore_authentication"));
      if (result.error) {
        console.log("failed to restore authentication state", result);
        return false;
      } else {
        user$.next(result.user);
        billingAccount$.next(result.billing_account);
        return true;
      }
    } else {
      return true;
    }
  } else {
    return false;
  }
}
