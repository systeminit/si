import assert from "node:assert";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import JWT from "npm:jsonwebtoken";
import { createPrivateKey } from "node:crypto";

const SDF_LOCATION = Deno.env.get('SDF_LOCATION');

let _JWT_PRIVATE_KEY: string = Deno.env.get('JWT_PRIVATE_KEY');
if (!_JWT_PRIVATE_KEY && Deno.env.get('JWT_PRIVATE_KEY_PATH')) {
  // path is relative to .env file
  _JWT_PRIVATE_KEY = Deno.readFileSync(`${Deno.env.get('JWT_PRIVATE_KEY_PATH')}`, 'utf-8');
}

assert(_JWT_PRIVATE_KEY, "Key Required");

_JWT_PRIVATE_KEY = createPrivateKey(_JWT_PRIVATE_KEY);

type SdfAuthTokenData = {
  user_pk: string;
  workspace_pk: string;
};

async function createJWT(
  payload: Record<string, any>,
  options?: Omit<JWT.SignOptions, 'algorithm'>,
) {
  return JWT.sign(payload, _JWT_PRIVATE_KEY!, { algorithm: "RS256", ...options });
}

async function smoke_test_sdf(userId: string, workspaceId: string) {
  const payload: SdfAuthTokenData = {
    user_pk: userId,
    workspace_pk: workspaceId,
  };
  // can add more metadata, expiry, etc...
  const token = await createJWT(payload, { subject: userId });
  const headers = {
    "Content-Type": "application/json",
    "Authorization": `Bearer ${token}`,
    "Cache-Control": "no-cache",
  };

  const rUlid = new URLSearchParams({ requestUlid: ulid() })
  const resp = await fetch(`${SDF_LOCATION}/api/change_set/list_open_change_sets?${rUlid}`, { headers });
  if (!resp.ok) throw new Error(`Error ${resp.status}: ${await resp.text()}`);
  const data = await resp.json();
  assert(data.headChangeSetId, "Expected headChangeSetId");
  const head = data.changeSets.find((c) => c.name === "HEAD");
  assert(head, "Expected a HEAD changeset")
  assert(head.id === data.headChangeSetId, "Expected HEAD changesets to match headChangeSetId");

  console.log("~~ SUCCESS ~~")
  console.log("/ fin");
}

if (import.meta.main) {
  assert(Deno.args.length === 2, "workspaceId and userId argument required");
  const workspaceId = Deno.args[0];
  const userId = Deno.args[1];
  await smoke_test_sdf(userId, workspaceId);
}

// example invocation:
// SDF_LOCATION=http://localhost:8080 JWT_PRIVATE_KEY=$JWT_PRIVATE_KEY deno run --allow-net --allow-env main.ts 01HRFEV0S23R1G23RP75QQDCA7 01HRFEV0RMWMH5SGBGDARH3G48
