import { registry } from "@/registry";
import { EntityObject } from "@/systemComponent";
import {
  ResourceHealth,
  ResourceStatus,
  SyncResourceRequest,
  SyncResourceReply,
  CalculatePropertiesRequest,
  CalculatePropertiesResult,
} from "../../veritech/intelligence";
import { Event } from "../../veritech/eventLog";
import { siExec } from "../siExec";
import {
  AwsCliEnv,
  awsCredential,
  awsKubeConfig,
  awsRegion,
} from "./awsShared";
import _ from "lodash";

const intelligence = (registry.get("awsEks") as EntityObject).intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  const result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {},
    },
  };
  for (const pred of req.predecessors) {
    if (pred.entity.objectType == "kubernetesCluster") {
      _.set(
        result.inferredProperties,
        ["__baseline", "clusterName"],
        pred.entity.name,
      );
    }
  }
  return result;
};

intelligence.syncResource = async function(
  request: SyncResourceRequest,
  event: Event,
): Promise<SyncResourceReply> {
  const awsCredResult = awsCredential(request);
  if (awsCredResult.syncResourceReply) {
    return awsCredResult.syncResourceReply;
  }
  let awsEnv: AwsCliEnv;
  if (awsCredResult.awsCliEnv) {
    awsEnv = awsCredResult.awsCliEnv as AwsCliEnv;
  } else {
    throw new Error("aws cli function didn't return an environment");
  }

  let region: string;
  const awsRegionResult = awsRegion(request);
  if (awsRegionResult.syncResourceReply) {
    return awsRegionResult.syncResourceReply;
  }
  if (awsRegionResult.region) {
    region = awsRegionResult.region;
  } else {
    throw new Error("aws node didn't have a region set");
  }

  const awsCmd = await siExec(
    event,
    "aws",
    [
      "eks",
      "describe-cluster",
      "--region",
      region,
      "--name",
      request.entity.properties.__baseline.clusterName,
    ],
    {
      reject: false,
      env: awsEnv,
    },
  );

  // If the describe-cluster failed, early return
  if (awsCmd.failed) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "aws eks describe-cluster failed",
      errorOutput: awsCmd.stderr,
    };

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  }

  const awsKubeConfigResult = await awsKubeConfig(request, event, awsEnv);
  if (awsKubeConfigResult.syncResourceReply) {
    return awsKubeConfigResult.syncResourceReply;
  }
  if (!awsKubeConfigResult.kubeconfig) {
    throw new Error("no reply or resource for aws kube config");
  }
  const kubeconfigPath = awsKubeConfigResult.kubeconfig;

  const kubectlVersionCmd = await siExec(
    event,
    "kubectl",
    ["version", "--kubeconfig", kubeconfigPath, "--output", "json"],
    {
      reject: false,
      env: awsEnv,
    },
  );

  // If kubectl version failed, early return
  if (kubectlVersionCmd.failed) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "kubectl version command failed",
      errorOutput: kubectlVersionCmd.stderr,
    };

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  }

  const awsJson = JSON.parse(awsCmd.stdout);
  const kubectlVersionJson = JSON.parse(kubectlVersionCmd.stdout);

  const reply: SyncResourceReply = {
    resource: {
      state: {
        data: {
          aws: awsJson,
          kubeconfig: "generated",
          kubernetes: kubectlVersionJson,
        },
      },
      health: ResourceHealth.Ok,
      status: ResourceStatus.Created,
    },
  };
  return reply;
};
