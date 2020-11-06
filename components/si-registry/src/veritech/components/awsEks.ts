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
import { promises as fs } from "fs";
import os from "os";
import path from "path";

import { awsCredential, AwsCliEnv, awsKubeConfig } from "./awsShared";

const awsEks = registry.get("awsEks") as EntityObject;
const intelligence = awsEks.intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  console.log(`calulating properties for awsEks`, { req });
  console.dir(req, { depth: Infinity });

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
): Promise<SyncResourceReply> {
  console.log(`syncing awsEks`);
  console.dir(request, { depth: Infinity });

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

  const awsCmd = await execa(
    "aws",
    [
      "eks",
      "describe-cluster",
      "--region",
      request.entity.properties.__baseline.region,
      "--name",
      request.entity.properties.__baseline.clusterName,
    ],
    {
      env: awsEnv,
    },
  );
  if (awsCmd.failed) {
    const reply: SyncResourceReply = {
      resource: {
        state: {
          data: request.resource.state?.data,
          errorMsg: "aws eks describe-cluster failed",
          errorOutput: awsCmd.stderr,
        },
        health: ResourceHealth.Ok,
        status: ResourceStatus.Created,
      },
    };
    return reply;
  }

  const awsKubeConfigResult = await awsKubeConfig(request, awsEnv);
  if (awsKubeConfigResult.syncResourceReply) {
    return awsKubeConfigResult.syncResourceReply;
  }
  if (!awsKubeConfigResult.kubeconfig) {
    throw new Error("no reply or resource for aws kube config");
  }
  const kubeconfigPath = awsKubeConfigResult.kubeconfig;

  const kubectlVersionCmd = await execa(
    "kubectl",
    ["version", "--kubeconfig", kubeconfigPath, "--output", "json"],
    {
      env: awsEnv,
    },
  );
  if (kubectlVersionCmd.failed) {
    const reply: SyncResourceReply = {
      resource: {
        state: {
          data: request.resource.state?.data,
          errorMsg: "kubectl version command failed",
          errorOutput: kubectlVersionCmd.stderr,
        },
        health: ResourceHealth.Ok,
        status: ResourceStatus.Created,
      },
    };
    return reply;
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
