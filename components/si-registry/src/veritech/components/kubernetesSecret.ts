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

const kubernetesSecret = registry.get("kubernetesSecret") as EntityObject;
const intelligence = kubernetesSecret.intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  console.log(`calulating properties for kubernetesSecret`, { req });
  console.dir(req, { depth: Infinity });

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
