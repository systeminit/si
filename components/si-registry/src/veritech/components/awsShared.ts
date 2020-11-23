import {
  Entity,
  SyncResourceReply,
  ResourceHealth,
  ResourceStatus,
} from "../intelligence";
import { Event } from "../../veritech/eventLog";
import { siExec } from "../siExec";
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
  };
}

export interface AwsCredentialResult {
  syncResourceReply?: SyncResourceReply;
  awsCliEnv?: AwsCliEnv;
}

export function awsCredential(request: AwsInputRequest): AwsCredentialResult {
  const awsAccessKeyCredential = _.find(request.predecessors, [
    "entity.objectType",
    "awsAccessKeyCredential",
  ]);

  if (!awsAccessKeyCredential) {
    const reply: SyncResourceReply = {
      resource: {
        state: {
          data: request.resource.state?.data,
          errorMsg: "No awsAccessKeyCredential attached!",
        },
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
    return { syncResourceReply: reply };
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
  let clusterName: string;
  if (request.entity.objectType == "awsEks") {
    region = request.entity.properties.__baseline.region;
    clusterName = request.entity.properties.__baseline.clusterName;
  } else {
    const awsEksPred = _.find(request.predecessors, [
      "entity.objectType",
      "awsEks",
    ]);
    if (awsEksPred) {
      region = awsEksPred.entity.properties.__baseline.region;
      clusterName = awsEksPred.entity.properties.__baseline.clusterName;
    } else {
      const reply: SyncResourceReply = {
        resource: {
          state: {
            data: request.resource.state?.data,
            errorMsg: "aws eks update-kubeconfig failed",
            errorOutput: "no awsEks entity attached!",
          },
          health: ResourceHealth.Error,
          status: ResourceStatus.Failed,
        },
      };
      return { syncResourceReply: reply };
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
    const reply: SyncResourceReply = {
      resource: {
        state: {
          data: request.resource.state?.data,
          errorMsg: "aws eks update-kubeconfig failed",
          errorOutput: awsKubeConfigCmd.stderr,
        },
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
    return { syncResourceReply: reply };
  }

  return { kubeconfig: kubeconfigPath };
}
