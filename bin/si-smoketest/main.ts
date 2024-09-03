import assert from "node:assert";
import JWT from "npm:jsonwebtoken";
import { createPrivateKey } from "node:crypto";
import { SdfApiClient } from "./sdf_api_client.ts";

if (import.meta.main) {
  assert(Deno.env.get("SDF_API_URL")?.length > 0, "Expected SDF_API_URL env var");
  assert(Deno.env.get("AUTH_API_URL")?.length > 0, "Expected AUTH_API_URL env var");

  assert([2, 3].includes(Deno.args.length), "Expected args: workspaceId, userEmail and userPassword (optional)");
  const workspaceId = Deno.args[0];
  const userId = Deno.args[1];
  const password = Deno.args[2];
  const sdf = await SdfApiClient.init(
    workspaceId,
    userId,
    password
  );
  await smoke_test_sdf(sdf);
}

async function smoke_test_sdf(sdf: SdfApiClient) {
  const resp = await sdf.fetch("/change_set/list_open_change_sets");
  console.log(resp.status);
  if (!resp.ok) throw new Error(`Error ${resp.status}: ${await resp.text()}`);
  const data = await resp.json();
  // TODO finish this test
  console.log(data);
  assert(data.headChangeSetId, "Expected headChangeSetId");
  const head = data.changeSets.find((c) => c.id === data.headChangeSetId);
  assert(head, "Expected a HEAD changeset");


  console.log("~~ SUCCESS ~~");
}

