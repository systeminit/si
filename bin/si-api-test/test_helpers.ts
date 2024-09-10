import { SdfApiClient } from "./sdf_api_client.ts";
import assert from "node:assert";

export function sleep(ms: number) {
  const natural_ms = Math.max(0, Math.floor(ms));
  console.log(`Sleeping for ${natural_ms} ms`);
  return new Promise((resolve) => setTimeout(resolve, natural_ms));
}

// Run fn n times, with increasing intervals between tries
export async function retryWithBackoff(
  fn: () => Promise<void>,
  retries = 60,
  backoffFactor = 1.2,
  initialDelay = 2, /// in seconds
) {
  const maxDelay = 10;
  let latest_err;
  let try_count = 0;
  let delay = initialDelay;

  console.log("Running retry_with_backoff block");
  do {
    try_count++;
    latest_err = undefined;
    console.log(`try number ${try_count}`);

    try {
      await fn();
    } catch (e) {
      latest_err = e;
      await sleep(delay * 1000);
      delay = Math.min(delay * backoffFactor, maxDelay);
    }
  } while (latest_err && try_count < retries);

  if (latest_err) {
    throw latest_err;
  }
}

export async function runWithTemporaryChangeset(
  sdf: SdfApiClient,
  fn: (sdf: SdfApiClient, changesetId: string) => Promise<void>,
) {
  // CREATE CHANGESET
  const startTime = new Date();
  const changeSetName = `API_TEST - ${startTime.toISOString()}`;

  const data = await sdf.call({
    route: "create_change_set",
    body: {
      changeSetName,
    },
  });
  assert(typeof data.changeSet === "object", "Expected changeSet in response");
  const changeSet = data.changeSet;
  assert(changeSet?.id, "Expected Change Set id");
  assert(
    changeSet?.name === changeSetName,
    `Changeset name should be ${changeSetName}`,
  );
  const changeSetId = changeSet.id;

  // RUN FN
  let err;
  try {
    await fn(sdf, changeSetId);
  } catch (e) {
    err = e;
  }

  // DELETE CHANGE SET
  await sdf.call({
    route: "abandon_change_set",
    body: {
      changeSetId,
    },
  });

  if (err) {
    throw err;
  }
}
