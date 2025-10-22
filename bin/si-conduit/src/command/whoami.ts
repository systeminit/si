import { Configuration, WhoamiApi } from "@systeminit/api-client";

export async function callWhoami(apiConfiguration: Configuration) {
  const whoamiApi = new WhoamiApi(apiConfiguration);

  const result = await whoamiApi.whoami();
  console.log(JSON.stringify(result.data, null, 2));
}
