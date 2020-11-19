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
import { kubernetesSync, kubernetesApply } from "./kubernetesShared";
import _ from "lodash";

const intelligence = (registry.get("kubernetesDeployment") as EntityObject)
  .intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
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
  const imagePullSecrets = [];
  for (const pred of req.predecessors) {
    if (pred.entity.objectType == "kubernetesSecret") {
      if (
        pred.entity.properties.__baseline.kubernetesObject?.type ==
        "kubernetes.io/dockerconfigjson"
      ) {
        imagePullSecrets.push({
          name:
            pred.entity.properties.__baseline.kubernetesObject.metadata?.name,
        });
      }
    } else if (pred.entity.objectType == "dockerImage") {
      if (
        !Array.isArray(
          result.inferredProperties.__baseline.kubernetesObject?.spec?.template
            ?.spec?.containers,
        )
      ) {
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
      // add the image pull secret
    }
    if (pred.entity.objectType == "kubernetesNamespace") {
      if (pred.entity.properties.__baseline.kubernetesObject?.metadata?.name) {
        _.set(
          result.inferredProperties,
          ["__baseline", "kubernetesObject", "metadata", "namespace"],
          pred.entity.properties.__baseline.kubernetesObject.metadata.name,
        );
      }
    }
  }
  if (imagePullSecrets.length > 0) {
    _.set(
      result.inferredProperties,
      [
        "__baseline",
        "kubernetesObject",
        "spec",
        "template",
        "spec",
        "imagePullSecrets",
      ],
      imagePullSecrets,
    );
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
