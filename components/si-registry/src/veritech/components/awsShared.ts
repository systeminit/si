import {
  Entity,
  SyncResourceReply,
  ResourceHealth,
  ResourceStatus,
} from "../intelligence";
import { Event } from "../../veritech/eventLog";
import { siExec } from "../siExec";
import { failSyncResourceReply } from "../syncResource";
import _ from "lodash";
import { promises as fs } from "fs";
import os from "os";
import path from "path";

export interface AwsCliEnv {
  AWS_ACCESS_KEY_ID: string;
  AWS_SECRET_ACCESS_KEY: string;
  AWS_DEFAULT_OUTPUT: string;
  [key: string]: string;
}

interface AwsInputRequest {
  entity: Entity;
  predecessors: {
    entity: Entity;
  }[];
  resource: {
    state?: any;
    health: ResourceHealth;
    status: ResourceStatus;
  };
}

export interface AwsRegionResult {
  syncResourceReply?: SyncResourceReply;
  region?: string;
}

export interface AwsCredentialResult {
  syncResourceReply?: SyncResourceReply;
  awsCliEnv?: AwsCliEnv;
}

export function awsRegion(request: AwsInputRequest): AwsRegionResult {
  const aws = _.find(request.predecessors, ["entity.objectType", "aws"]);

  if (!aws) {
    return {
      syncResourceReply: failSyncResourceReply(request, {
        errorMsg: "No aws attached!",
      }),
    };
  }

  return { region: aws.entity.properties.__baseline.region };
}

export function awsCredential(request: AwsInputRequest): AwsCredentialResult {
  const awsAccessKeyCredential = _.find(request.predecessors, [
    "entity.objectType",
    "awsAccessKeyCredential",
  ]);

  if (!awsAccessKeyCredential) {
    return {
      syncResourceReply: failSyncResourceReply(request, {
        errorMsg: "No awsAccessKeyCredential attached!",
      }),
    };
  }

  const awsCliEnv = {
    AWS_ACCESS_KEY_ID:
      awsAccessKeyCredential?.entity.properties.__baseline.decrypted
        ?.accessKeyId,
    AWS_SECRET_ACCESS_KEY:
      awsAccessKeyCredential?.entity.properties.__baseline.decrypted?.secretKey,
    AWS_DEFAULT_OUTPUT: "json",
  };
  return { awsCliEnv };
}

export interface AwsKubeConfigResult {
  syncResourceReply?: SyncResourceReply;
  kubeconfig?: string;
}

export async function awsKubeConfig(
  request: AwsInputRequest,
  event: Event,
  awsEnv: AwsCliEnv,
): Promise<AwsKubeConfigResult> {
  const tempdir = await fs.mkdtemp(path.join(os.tmpdir(), "kubeconfig-"));
  const kubeconfigPath = path.join(tempdir, "config");

  let region: string;
  const awsRegionResult = awsRegion(request);
  if (awsRegionResult.syncResourceReply) {
    return { syncResourceReply: awsRegionResult.syncResourceReply };
  }
  if (awsRegionResult.region) {
    region = awsRegionResult.region;
  } else {
    throw new Error("aws node didn't have a region set");
  }

  let clusterName: string;
  if (request.entity.objectType == "awsEks") {
    clusterName = request.entity.properties.__baseline.clusterName;
  } else {
    const awsEksPred = _.find(request.predecessors, [
      "entity.objectType",
      "awsEks",
    ]);
    if (awsEksPred) {
      clusterName = awsEksPred.entity.properties.__baseline.clusterName;
    } else {
      return {
        syncResourceReply: failSyncResourceReply(request, {
          errorMsg: "aws eks update-kubeconfig failed",
          errorOutput: "no awsEks entity attached!",
          health: ResourceHealth.Error,
          status: ResourceStatus.Failed,
        }),
      };
    }
  }

  const awsKubeConfigCmd = await siExec(
    event,
    "aws",
    [
      "eks",
      "--region",
      region,
      "update-kubeconfig",
      "--name",
      clusterName,
      "--kubeconfig",
      kubeconfigPath,
    ],
    {
      reject: false,
      env: awsEnv,
    },
  );

  if (awsKubeConfigCmd.failed) {
    return {
      syncResourceReply: failSyncResourceReply(request, {
        errorMsg: "aws eks update-kubeconfig failed",
        errorOutput: awsKubeConfigCmd.stderr,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      }),
    };
  }

  return { kubeconfig: kubeconfigPath };
}
