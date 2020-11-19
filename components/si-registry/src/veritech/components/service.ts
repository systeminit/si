import { registry } from "@/registry";
import { EntityObject } from "@/systemComponent";
import {
  ActionRequest,
  ActionReply,
  CalculatePropertiesRequest,
  CalculatePropertiesResult,
  ResourceHealth,
  ResourceStatus,
  SyncResourceRequest,
  SyncResourceReply,
} from "../../veritech/intelligence";
import { kubernetesApplyOrder } from "./kubernetesShared";
import _ from "lodash";

const intelligence = (registry.get("service") as EntityObject).intelligence;

intelligence.calculateProperties = function(
  _req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  const result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {},
    },
  };

  return result;
};

intelligence.syncResource = async function(
  _request: SyncResourceRequest,
): Promise<SyncResourceReply> {
  const state = {};

  return {
    resource: {
      state,
      health: ResourceHealth.Ok,
      status: ResourceStatus.Created,
    },
  };
};

intelligence.actions = {
  async deploy(request: ActionRequest): Promise<ActionReply> {
    const actions = _.filter(request.successors, s =>
      kubernetesApplyOrder.includes(s.entity.objectType),
    );
    // Sort the actions in the order of a safe apply order.
    // Thanks to Stack Overflow, hope I've passed the audition!
    // See: https://stackoverflow.com/a/44063445
    actions.sort(
      (a, b) =>
        kubernetesApplyOrder.indexOf(a.entity.objectType) -
        kubernetesApplyOrder.indexOf(b.entity.objectType),
    );

    const reply: ActionReply = {
      resource: {
        state: {
          deployedTs: Math.floor(+new Date() / 1000),
        },
        health: ResourceHealth.Ok,
        status: ResourceStatus.Created,
      },
      actions: actions.map(s => ({ entityId: s.entity.id, action: "apply" })),
    };
    return reply;
  },
};
