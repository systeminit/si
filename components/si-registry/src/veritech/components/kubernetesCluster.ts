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

const kubernetesCluster = registry.get("kubernetesCluster") as EntityObject;
const intelligence = kubernetesCluster.intelligence;

intelligence.syncResource = async function(
  request: SyncResourceRequest,
): Promise<SyncResourceReply> {
  console.log(`syncing kubernetes cluster`);
  console.dir(request, { depth: Infinity });

  const kubectlConfigView = await execa("kubectl", [
    "config",
    "view",
    "-o",
    "json",
  ]);
  if (kubectlConfigView.failed) {
    const reply: SyncResourceReply = {
      resource: {
        state: {
          data: request.resource.state,
          errorMsg: kubectlConfigView.stderr,
        },
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
    return reply;
  } else {
    const kubectlConfigData = JSON.parse(kubectlConfigView.stdout);
    const reply: SyncResourceReply = {
      resource: {
        state: {
          data: kubectlConfigData,
        },
        health: ResourceHealth.Ok,
        status: ResourceStatus.Created,
      },
    };
    return reply;
  }
};
