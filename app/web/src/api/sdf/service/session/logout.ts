import Bottle from "bottlejs";
import { SDF } from "@/api/sdf";
import { user$ } from "@/observable/user";
import { billingAccount$ } from "@/observable/billing_account";

export async function logout() {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  sdf.token = undefined;
  //if (sdf.update) {
  //  sdf.update.socket.close();
  //}
  user$.next(null);
  billingAccount$.next(null);
  sessionStorage.clear();
}
