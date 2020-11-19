import { registry } from "@/registry";
import { EntityObject } from "@/systemComponent";
import {
  ResourceHealth,
  ResourceStatus,
  SyncResourceRequest,
  SyncResourceReply,
} from "../../veritech/intelligence";
import { Event } from "../../veritech/eventLog";
import { siExec } from "../siExec";

const intelligence = (registry.get("kubernetesCluster") as EntityObject)
  .intelligence;

intelligence.syncResource = async function(
  request: SyncResourceRequest,
  event: Event,
): Promise<SyncResourceReply> {
  const kubectlConfigView = await siExec(
    event,
    "kubectl",
    ["config", "view", "-o", "json"],
    {
      reject: false,
    },
  );

  // If kubectl config failed, early return
  if (kubectlConfigView.failed) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "kubectl config command failed",
      errorOutput: kubectlConfigView.stderr,
    };

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  }

  const kubectlConfigData = JSON.parse(kubectlConfigView.stdout);

  return {
    resource: {
      state: {
        data: kubectlConfigData,
      },
      health: ResourceHealth.Ok,
      status: ResourceStatus.Created,
    },
  };
};
