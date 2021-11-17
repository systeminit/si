import Bottle from "bottlejs";
import { SDF } from "@/api/sdf";

export async function logout() {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  sdf.token = undefined;
  if (sdf.ws) {
    sdf.ws.socket.close();
  }
  sessionStorage.clear();
  // We reload the window because we want to reset the state!
  window.location.reload();
}
