import { WhoamiApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import { ApiContext } from "../api.ts";

export async function callWhoami(ctx: Context, apiCtx: ApiContext) {
  const whoamiApi = new WhoamiApi(apiCtx.config);

  const result = await whoamiApi.whoami();
  console.log(JSON.stringify(result.data, null, 2));
}
