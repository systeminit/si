import { registry } from "@/registry";
import { EntityObject } from "@/systemComponent";
import {
  ActionRequest,
  ActionReply,
  SyncResourceRequest,
  SyncResourceReply,
  CalculatePropertiesRequest,
  CalculatePropertiesResult,
} from "../../veritech/intelligence";
import { Event } from "../../veritech/eventLog";
import {
  kubernetesNamespaceProperties,
  kubernetesSync,
  kubernetesApply,
} from "./kubernetesShared";
import _ from "lodash";

const intelligence = (registry.get("kubernetesService") as EntityObject)
  .intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
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
  event: Event,
): Promise<SyncResourceReply> {
  return await kubernetesSync(request, event);
};

intelligence.actions = {
  async apply(request: ActionRequest, event: Event): Promise<ActionReply> {
    return await kubernetesApply(request, event);
  },
};
