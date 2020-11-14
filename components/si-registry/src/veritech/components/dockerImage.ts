import { registry } from "@/registry";
import { EntityObject } from "@/systemComponent";
import {
  ActionRequest,
  ActionReply,
  ResourceHealth,
  ResourceStatus,
  SyncResourceRequest,
  SyncResourceReply,
  CalculatePropertiesRequest,
  CalculatePropertiesResult,
} from "../../veritech/intelligence";
import execa from "execa";
import { promises as fs } from "fs";
import os from "os";
import path from "path";

const dockerImage = registry.get("dockerImage") as EntityObject;
const intelligence = dockerImage.intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  console.log(`calulating properties`, { req });
  console.dir(req, { depth: Infinity });

  const result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {},
    },
  };
  for (const pred of req.predecessors) {
    if (pred.entity.objectType == "service") {
      result.inferredProperties.__baseline["image"] = pred.entity.name;
    }
  }
  return result;
};

intelligence.syncResource = async function(
  request: SyncResourceRequest,
): Promise<SyncResourceReply> {
  console.log(`syncing image`);
  console.dir(request, { depth: Infinity });
  const state: Record<string, any> = {};

  let tempdir;
  for (const pred of request.predecessors) {
    if (pred.entity.objectType == "dockerHubCredential") {
      tempdir = await fs.mkdtemp(path.join(os.tmpdir(), "docker-"));
      const creds = pred.entity.properties.__baseline.decrypted;
      const auth = Buffer.from(
        `${creds?.username}:${creds?.password}`,
      ).toString("base64");
      const config = {
        auths: {
          "https://index.docker.io/v1/": {
            auth,
          },
        },
      };
      await fs.writeFile(
        path.join(tempdir, "config.json"),
        JSON.stringify(config, null, 0),
        { mode: 0o400 },
      );
    }
  }

  // Pull the image with `docker image pull`
  let pullArgs: string[] = [];
  if (tempdir) {
    pullArgs = pullArgs.concat(["--config", tempdir]);
  }
  pullArgs = pullArgs.concat([
    "image",
    "pull",
    request.entity.properties.__baseline.image,
  ]);
  console.log(`running command; cmd="docker ${pullArgs.join(" ")}"`);
  const dockerImagePull = await execa("docker", pullArgs, { all: true });

  // If the image pull failed, early return
  if (dockerImagePull.failed) {
    state["data"] = request.resource.state;
    state["errorMsg"] = dockerImagePull.stderr;

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  }

  // Inspect the now-local image
  let inspectArgs: string[] = [];
  if (tempdir) {
    inspectArgs = inspectArgs.concat(["--config", tempdir]);
  }
  inspectArgs = inspectArgs.concat([
    "image",
    "inspect",
    request.entity.properties.__baseline.image,
  ]);
  console.log(`running command; cmd="docker ${inspectArgs.join(" ")}"`);
  const dockerImageInspect = await execa("docker", inspectArgs, { all: true });

  // If the image inspect failed, early return
  if (dockerImagePull.failed) {
    state["data"] = request.resource.state;
    state["errorMsg"] = dockerImageInspect.stderr;

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  }

  // Set state data
  const dockerImageJson = JSON.parse(dockerImageInspect.stdout);
  state["data"] = dockerImageJson[0];

  return {
    resource: {
      state,
      health: ResourceHealth.Ok,
      status: ResourceStatus.Created,
    },
  };
};

intelligence.actions = {
  async deploy(request: ActionRequest): Promise<ActionReply> {
    const actions: ActionReply["actions"] = [];
    for (const child of request.successors) {
      if (child.entity.objectType == "service") {
        actions.push({ action: "deploy", entityId: child.entity.id });
      }
    }
    const reply: ActionReply = {
      resource: {
        state: {
          alex: "van halen",
          deployedBy: request.predecessors.map(p => p.entity.name),
        },
        health: ResourceHealth.Ok,
        status: ResourceStatus.Created,
      },
      actions,
    };
    return reply;
  },
};
