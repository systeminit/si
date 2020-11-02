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
import { kubernetesNamespaceProperties } from "./kubernetesShared";
import _ from "lodash";
import execa from "execa";

const kubernetesService = registry.get("kubernetesService") as EntityObject;
const intelligence = kubernetesService.intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  console.log(`calulating properties for kubernetesService`, { req });
  console.dir(req, { depth: Infinity });

  const result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {
        kubernetesObject: {
          apiVersion: "v1",
          kind: "Service",
          metadata: {
            name: `${req.entity.name}-service`,
          },
        },
      },
    },
  };
  for (const pred of req.predecessors) {
    if (pred.entity.objectType == "kubernetesDeployment") {
      // First, assign the targeting
      if (
        pred.entity.properties.__baseline.kubernetesObject?.spec?.template
          ?.metadata?.labels?.app
      ) {
        const appName =
          pred.entity.properties.__baseline.kubernetesObject?.spec?.template
            ?.metadata?.labels?.app;
        _.set(
          result,
          [
            "inferredProperties",
            "__baseline",
            "kubernetesObject",
            "spec",
            "selector",
            "app",
          ],
          appName,
        );
      }
      // Second, assign the ports
      const containers = _.get(pred.entity.properties, [
        "__baseline",
        "kubernetesObject",
        "spec",
        "template",
        "spec",
        "containers",
      ]);
      if (containers) {
        const ports = [];
        for (const container of containers) {
          console.log("I am containing the shit out of you");
          console.dir(container);
          if (container.ports) {
            for (const port of container.ports) {
              if (port.containerPort) {
                const portSpec: Record<string, any> = {};
                portSpec["port"] = port.containerPort;
                if (port.protocol) {
                  portSpec["protocol"] = port.protocol;
                }
                if (container.name) {
                  portSpec["name"] = `${container.name}-${portSpec["port"]}`;
                } else {
                  portSpec["name"] = `port-${portSpec["port"]}`;
                }
                ports.push(portSpec);
              }
            }
          }
        }
        if (ports) {
          _.set(
            result,
            [
              "inferredProperties",
              "__baseline",
              "kubernetesObject",
              "spec",
              "ports",
            ],
            ports,
          );
        }
      }
    } else if (pred.entity.objectType == "kubernetesNamespace") {
      kubernetesNamespaceProperties(result, pred.entity);
    }
  }
  return result;
};

intelligence.syncResource = async function(
  request: SyncResourceRequest,
): Promise<SyncResourceReply> {
  console.log(`syncing kubernetes service`);
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
    console.log(`applying kubernetes service`);
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
