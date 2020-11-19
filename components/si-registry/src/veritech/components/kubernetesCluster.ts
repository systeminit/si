import { registry } from "@/registry";
import { EntityObject } from "@/systemComponent";
import {
  ResourceHealth,
  ResourceStatus,
  SyncResourceRequest,
  SyncResourceReply,
} from "../../veritech/intelligence";
import execa from "execa";

const intelligence = (registry.get("kubernetesCluster") as EntityObject)
  .intelligence;

intelligence.syncResource = async function(
  request: SyncResourceRequest,
): Promise<SyncResourceReply> {
  const kubectlArgs = ["config", "view", "-o", "json"];
  console.log(`running command; cmd="kubectl ${kubectlArgs.join(" ")}"`);
  const kubectlConfigView = await execa("kubectl", kubectlArgs, {
    reject: false,
  });

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
