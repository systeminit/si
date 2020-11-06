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
import { kubernetesApply, kubernetesSync } from "./kubernetesShared";

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
  return await kubernetesSync(request);
};

intelligence.actions = {
  async apply(request: ActionRequest): Promise<ActionReply> {
    return await kubernetesApply(request);
  },
};
