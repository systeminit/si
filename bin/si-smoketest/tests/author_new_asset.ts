import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";

export default async function author_new_asset(sdfApiClient: SdfApiClient) {

  // Move into a new changeset
  const startTime = new Date();
  const changeSetName = `API_TEST author_new_asset - ${startTime.toISOString()}`;

  // CREATE CHANGE SET
  const data = await sdfApiClient.createChangeSet(changeSetName);
  assert(typeof data.changeSet === "object", "Expected changeSet in response");
  const changeSet = data.changeSet;
  assert(changeSet?.id, "Expected Change Set id");
  assert(changeSet?.name === changeSetName, `Changeset name should be ${changeSetName}`);
  const changeSetId = changeSet.id;

  // Actually create a new component
  const createNewAsset = {
    visibility_change_set_pk: changeSetId,
    name: `api-test-${startTime.toISOString()}`,
    color: "#FF0000",
  };
  const createComponentResp = await sdfApiClient.createComponent(createComponentPayload);
  const newComponentId = createComponentResp?.componentId;
  assert(newComponentId, "Expected to get a component id after creation");

  // DELETE CHANGE SET
  await sdfApiClient.abandonChangeSet(changeSetId);

}


/*

fetch("https://app.systeminit.com/api/variant/create_variant", {
  "headers": {
    "accept": "application/json, text/plain, */*",
    "accept-language": "en-GB,en-US;q=0.9,en;q=0.8",
    "authorization": "Bearer",
    "content-type": "application/json",
    "priority": "u=1, i",
    "sec-ch-ua": "\"Chromium\";v=\"128\", \"Not;A=Brand\";v=\"24\", \"Google Chrome\";v=\"128\"",
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": "\"macOS\"",
    "sec-fetch-dest": "empty",
    "sec-fetch-mode": "cors",
    "sec-fetch-site": "same-origin",
    "traceparent": "00-68849a9aa8f7d69d769fb078a8ace767-4df5c55d24c5bec8-01",
    "cookie": "s",
    "Referer": "https://app.systeminit.com/w/01J4JCHSAM1BH0CH8HFHFDP9D2/01J712E4BSR0APXJJ5BV3STC7C/l/a/",
    "Referrer-Policy": "strict-origin-when-cross-origin"
  },
  "body": "{\"visibility_change_set_pk\":\"01J712E4BSR0APXJJ5BV3STC7C\",\"name\":\"Test Asset\",\"color\":\"#FF0000\",\"requestUlid\":\"01J712EBA4DFD5M1Q9EWB7K7WY\"}",
  "method": "POST"
});
/*