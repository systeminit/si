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
  const dockerImagePullProc = await execa(
    "docker",
    ["image", "pull", request.entity.properties.__baseline.image],
    { all: true },
  );
  const state: Record<string, any> = {};
  const dockerImageInspect = await execa("docker", [
    "image",
    "inspect",
    request.entity.properties.__baseline.image,
  ]);
  let health: ResourceHealth = ResourceHealth.Ok;
  if (dockerImageInspect.failed) {
    health = ResourceHealth.Error;
    state["data"] = request.resource.state;
    state["errorMsg"] = dockerImageInspect.stderr;
  } else {
    const dockerImageJson = JSON.parse(dockerImageInspect.stdout);
    state["data"] = dockerImageJson[0];
  }
  console.log("docker image pull", dockerImagePullProc.all);
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
