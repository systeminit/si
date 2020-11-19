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
import { Event } from "../../veritech/eventLog";
import _ from "lodash";

const intelligence = (registry.get("application") as EntityObject).intelligence;

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
  _event: Event,
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
  async deploy(request: ActionRequest, _event: Event): Promise<ActionReply> {
    const actions: ActionReply["actions"] = _.chain(request.successors)
      .filter(["entity.objectType", "service"])
      .map(s => ({ action: "deploy", entityId: s.entity.id }))
      .value();

    const state = {
      deployedTs: Math.floor(+new Date() / 1000),
    };

    const reply = {
      resource: {
        state,
        health: ResourceHealth.Ok,
        status: ResourceStatus.Created,
      },
      actions,
    };
    return reply;
  },
};
