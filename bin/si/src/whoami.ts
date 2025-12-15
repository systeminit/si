import { WhoamiApi } from "@systeminit/api-client";
import { Context } from "./context.ts";

export async function callWhoami() {
  const ctx = Context.instance();
  const whoamiApi = new WhoamiApi(Context.apiConfig());

  const result = await whoamiApi.whoami();
  console.log(JSON.stringify(result.data, null, 2));

  // Track whoami command
  ctx.analytics.trackEvent("whoami", {});
}
