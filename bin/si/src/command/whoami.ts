import { WhoamiApi } from "@systeminit/api-client";
import type { Context } from "../context.ts";
import type { ApiContext } from "../api.ts";

export async function callWhoami(_ctx: Context, apiCtx: ApiContext) {
  const whoamiApi = new WhoamiApi(apiCtx.config);

  const result = await whoamiApi.whoami();
  console.log(JSON.stringify(result.data, null, 2));
}
