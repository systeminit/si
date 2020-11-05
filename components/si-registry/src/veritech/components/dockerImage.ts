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
      console.dir({ creds });
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

  let args: string[] = [];
  if (tempdir) {
    args = args.concat(["--config", tempdir]);
  }
  args = args.concat([
    "image",
    "inspect",
    request.entity.properties.__baseline.image,
  ]);

  console.log(`about to exec: docker ${args.join(" ")}`);
  const dockerImageInspect = await execa("docker", args, { all: true });
  let health: ResourceHealth = ResourceHealth.Ok;
  if (dockerImageInspect.failed) {
    health = ResourceHealth.Error;
    state["data"] = request.resource.state;
    state["errorMsg"] = dockerImageInspect.stderr;
  } else {
    console.log("my outputs", { inspect: dockerImageInspect });
    const dockerImageJson = JSON.parse(dockerImageInspect.stdout);
    state["data"] = dockerImageJson[0];
  }
  console.log("docker image pull", dockerImageInspect.all);
  const reply: SyncResourceReply = {
    resource: {
      state,
      health,
      status: ResourceStatus.Created,
    },
  };
  return reply;
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
