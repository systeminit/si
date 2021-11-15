import { ReplaySubject } from "rxjs";
import { BillingAccount } from "@/api/sdf/dal/billing_account";
import { persistToSession } from "@/observable/session_state";

/**
 * The currently logged in billing account, or null if there isn't one.
 */
export const billingAccount$ = new ReplaySubject<BillingAccount | null>(1);
billingAccount$.next(null);
persistToSession("billingAccount", billingAccount$);
