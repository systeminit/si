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
import YAML from "yaml";
import execa from "execa";

const kubernetesDeployment = registry.get(
  "kubernetesDeployment",
) as EntityObject;
const intelligence = kubernetesDeployment.intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  console.log(`calulating properties for kubernetesDeployment`, { req });
  console.dir(req, { depth: Infinity });

  const result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {
        kubernetesObject: {
          apiVersion: "apps/v1",
          kind: "Deployment",
          metadata: {
            name: `${req.entity.name}-deployment`,
          },
          spec: {
            selector: {
              matchLabels: {
                app: `${req.entity.name}`,
              },
            },
            replicas: 1,
            template: {
              metadata: {
                labels: {
                  app: `${req.entity.name}`,
                },
              },
            },
          },
        },
      },
    },
  };
  for (const pred of req.predecessors) {
    if (pred.entity.objectType == "dockerImage") {
      if (
        !Array.isArray(
          result.inferredProperties.__baseline.kubernetesObject?.spec?.template
            ?.spec?.containers,
        )
      ) {
        console.log("I'm setting like a motherfucker");
        _.set(
          result.inferredProperties,
          [
            "__baseline",
            "kubernetesObject",
            "spec",
            "template",
            "spec",
            "containers",
          ],
          [],
        );
      }
      const containerSpec: Record<string, any> = {
        name: pred.entity.name,
        image: pred.entity.properties.__baseline["image"],
      };
      const ports = [];
      for (const resource of pred.resources) {
        if (resource.state["data"]) {
          for (const portString of Object.keys(
            resource.state.data?.Config?.ExposedPorts,
          )) {
            const portParts = portString.split("/");
            const port = parseInt(portParts[0], 10);
            const protocol = portParts[1].toUpperCase();
            if (port) {
              ports.push({
                containerPort: port,
                protocol: protocol,
              });
            }
          }
        }
      }
      if (ports.length > 0) {
        containerSpec.ports = ports;
      }

      result.inferredProperties.__baseline["kubernetesObject"]["spec"][
        "template"
      ]["spec"]["containers"].push(containerSpec);
    }
  }
  return result;
};

intelligence.syncResource = async function(
  request: SyncResourceRequest,
): Promise<SyncResourceReply> {
  console.log(`syncing kubernetes deployment`);
  console.dir(request, { depth: Infinity });

  const kubernetesCluster = _.find(request.predecessors, [
    "entity.objectType",
    "kubernetesCluster",
  ]);
  let currentContext = undefined;
  console.log("find me?", { kubernetesCluster });
  if (kubernetesCluster?.resource.state.data) {
    console.log("you get me");
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
    console.log(`applying kubernetes deployment`);
    console.dir(request, { depth: Infinity });
    const kubernetesCluster = _.find(request.predecessors, [
      "entity.objectType",
      "kubernetesCluster",
    ]);
    let currentContext = undefined;
    console.log("find me?", { kubernetesCluster });
    if (kubernetesCluster?.resource.state.data) {
      console.log("you get me");
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
