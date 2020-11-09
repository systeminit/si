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
import { kubernetesSync, kubernetesApply } from "./kubernetesShared";

const kubernetesService = registry.get("kubernetesService") as EntityObject;
const intelligence = kubernetesService.intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  console.log(`calulating properties for kubernetesService`, { req });
  console.dir(req, { depth: Infinity });

  let result: CalculatePropertiesResult = {
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
      result = kubernetesNamespaceProperties(result, pred.entity);
    }
  }
  return result;
};

intelligence.syncResource = async function(
  request: SyncResourceRequest,
): Promise<SyncResourceReply> {
  return await kubernetesSync(request);
};

intelligence.actions = {
  async apply(request: ActionRequest): Promise<ActionReply> {
    return await kubernetesApply(request);
  },
};
