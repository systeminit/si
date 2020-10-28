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
import _ from "lodash";
import execa from "execa";

const kubernetesNamespace = registry.get("kubernetesNamespace") as EntityObject;
const intelligence = kubernetesNamespace.intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  console.log(`calulating properties for kubernetesNamespace`, { req });
  console.dir(req, { depth: Infinity });

  const result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {
        kubernetesObject: {
          apiVersion: "v1",
          kind: "Namespace",
          metadata: {
            name: `${req.entity.name}`,
          },
        },
      },
    },
  };
  for (const pred of req.predecessors) {
    if (pred.entity.objectType == "application") {
      _.set(
        result.inferredProperties,
        ["__baseline", "kubernetesObject", "metadata", "name"],
        pred.entity.name,
      );
    }
  }
  return result;
};

intelligence.syncResource = async function(
  request: SyncResourceRequest,
): Promise<SyncResourceReply> {
  console.log(`syncing kubernetes namespace`);
  console.dir(request, { depth: Infinity });

  const kubernetesCluster = _.find(request.predecessors, [
    "entity.objectType",
    "kubernetesCluster",
  ]);
  let currentContext = undefined;
  if (kubernetesCluster?.resource.state.data) {
    currentContext = kubernetesCluster.resource.state.data["current-context"];
  }

  if (kubernetesCluster && currentContext) {
    const kubectlApply = await execa(
      "kubectl",
      [
        "apply",
        "-o",
        "json",
        "--context",
        currentContext,
        "--dry-run=server",
        "-f",
        "-",
      ],
      {
        input: request.entity.properties.__baseline["kubernetesObjectYaml"],
      },
    );
    if (kubectlApply.failed) {
      const reply: SyncResourceReply = {
        resource: {
          state: {
            data: request.resource.state?.data,
            errorMsg: "kubectl apply failed",
            errorOutput: kubectlApply.stderr,
          },
          health: ResourceHealth.Ok,
          status: ResourceStatus.Created,
        },
      };
      return reply;
    } else {
      const kubectlApplyJson = JSON.parse(kubectlApply.stdout);
      const reply: SyncResourceReply = {
        resource: {
          state: {
            data: kubectlApplyJson,
          },
          health: ResourceHealth.Ok,
          status: ResourceStatus.Created,
        },
      };
      return reply;
    }
  } else {
    const reply: SyncResourceReply = {
      resource: {
        state: {
          data: request.resource.state?.data,
          errorMsg: "No kubernetesCluster attached!",
        },
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
    return reply;
  }
};

intelligence.actions = {
  async apply(request: ActionRequest): Promise<ActionReply> {
    const actions: ActionReply["actions"] = [];
    console.log(`applying kubernetes namespace`);
    console.dir(request, { depth: Infinity });
    const kubernetesCluster = _.find(request.predecessors, [
      "entity.objectType",
      "kubernetesCluster",
    ]);
    let currentContext = undefined;
    if (kubernetesCluster?.resource.state.data) {
      currentContext = kubernetesCluster.resource.state.data["current-context"];
    }

    if (kubernetesCluster && currentContext) {
      const applyArgs = [
        "apply",
        "-o",
        "json",
        "--context",
        currentContext,
        "-f",
        "-",
      ];
      if (request.hypothetical) {
        applyArgs.push("--dry-run=server");
      }

      const kubectlApply = await execa("kubectl", applyArgs, {
        input: request.entity.properties.__baseline["kubernetesObjectYaml"],
      });
      if (kubectlApply.failed) {
        const reply: ActionReply = {
          resource: {
            state: {
              data: request.resource.state?.data,
              errorMsg: "kubectl apply failed",
              errorOutput: kubectlApply.stderr,
            },
            health: ResourceHealth.Ok,
            status: ResourceStatus.Created,
          },
          actions,
        };
        return reply;
      } else {
        const kubectlApplyJson = JSON.parse(kubectlApply.stdout);
        const reply: ActionReply = {
          resource: {
            state: {
              data: kubectlApplyJson,
            },
            health: ResourceHealth.Ok,
            status: ResourceStatus.Created,
          },
          actions,
        };
        return reply;
      }
    } else {
      const reply: ActionReply = {
        resource: {
          state: {
            data: request.resource.state?.data,
            errorMsg: "No kubernetesCluster attached!",
          },
          health: ResourceHealth.Error,
          status: ResourceStatus.Failed,
        },
        actions,
      };
      return reply;
    }
  },
};
