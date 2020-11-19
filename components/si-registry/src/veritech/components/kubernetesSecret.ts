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
import {
  kubernetesNamespaceProperties,
  kubernetesSync,
  kubernetesApply,
} from "./kubernetesShared";
import yaml from "yaml";

const intelligence = (registry.get("kubernetesSecret") as EntityObject)
  .intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  let result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {
        kubernetesObject: {
          apiVersion: "v1",
          kind: "Secret",
          type: "Opaque",
          metadata: {
            name: `${req.entity.name}`,
          },
        },
      },
    },
  };
  for (const pred of req.predecessors) {
    if (pred.entity.objectType == "kubernetesNamespace") {
      result = kubernetesNamespaceProperties(result, pred.entity);
    }
    if (pred.entity.objectType == "dockerHubCredential") {
      result.inferredProperties.__baseline.kubernetesObject["type"] =
        "kubernetes.io/dockerconfigjson";
      result.inferredProperties.__baseline.kubernetesObject["data"] = {
        ".dockerconfigjson": `$entity[${pred.entity.id}]`,
      };
    }
  }
  return result;
};

intelligence.syncResource = async function(
  request: SyncResourceRequest,
): Promise<SyncResourceReply> {
  for (const pred of request.predecessors) {
    if (pred.entity.objectType == "dockerHubCredential") {
      const creds = pred.entity.properties.__baseline.decrypted;
      const auth = Buffer.from(
        `${creds?.username}:${creds?.password}`,
      ).toString("base64");
      const dockerConfig = {
        auths: {
          "https://index.docker.io/v1/": {
            auth,
          },
        },
      };
      const dockerConfigBase64 = Buffer.from(
        JSON.stringify(dockerConfig),
      ).toString("base64");
      request.entity.properties.__baseline.kubernetesObject["data"] = {
        ".dockerconfigjson": dockerConfigBase64,
      };
      request.entity.properties.__baseline.kubernetesObjectYaml = yaml.stringify(
        request.entity.properties.__baseline.kubernetesObject,
      );
    }
  }
  return await kubernetesSync(request);
};

intelligence.actions = {
  async apply(request: ActionRequest): Promise<ActionReply> {
    for (const pred of request.predecessors) {
      if (pred.entity.objectType == "dockerHubCredential") {
        const creds = pred.entity.properties.__baseline.decrypted;
        const auth = Buffer.from(
          `${creds?.username}:${creds?.password}`,
        ).toString("base64");
        const dockerConfig = {
          auths: {
            "https://index.docker.io/v1/": {
              auth,
            },
          },
        };
        const dockerConfigBase64 = Buffer.from(
          JSON.stringify(dockerConfig),
        ).toString("base64");
        request.entity.properties.__baseline.kubernetesObject["data"] = {
          ".dockerconfigjson": dockerConfigBase64,
        };
        request.entity.properties.__baseline.kubernetesObjectYaml = yaml.stringify(
          request.entity.properties.__baseline.kubernetesObject,
        );
      }
    }
    return await kubernetesApply(request);
  },
};
