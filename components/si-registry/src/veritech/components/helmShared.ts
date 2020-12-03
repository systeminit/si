import {
  awsCredential,
  awsKubeConfig,
  AwsCliEnv,
  AwsInputRequest,
} from "./awsShared";
import { Event } from "../../veritech/eventLog";
import {
  SyncResourceReply,
  ResourceHealth,
  ResourceStatus,
  findEntityByType,
} from "../intelligence";
import { siExec, SiExecResult } from "../siExec";
import { promises as fs } from "fs";
import os from "os";
import path from "path";
import { XOR } from "ts-essentials";

interface HelmFailureResult {
  failure: SyncResourceReply;
}

interface HelmKubeConfigResponse {
  kubeconfigPath: string;
  awsEnv: AwsCliEnv;
}
type HelmKubeConfigResult = XOR<HelmFailureResult, HelmKubeConfigResponse>;

export async function helmKubeConfig(
  request: AwsInputRequest,
  event: Event,
): Promise<HelmKubeConfigResult> {
  const awsCredResult = awsCredential(request);
  if (awsCredResult.syncResourceReply) {
    return { failure: awsCredResult.syncResourceReply };
  } else if (!awsCredResult.awsCliEnv) {
    throw new Error("aws cli function didn't return an environment");
  }
  const awsEnv: AwsCliEnv = awsCredResult.awsCliEnv;
  const awsKubeConfigResult = await awsKubeConfig(request, event, awsEnv);
  if (awsKubeConfigResult.syncResourceReply) {
    return { failure: awsKubeConfigResult.syncResourceReply };
  } else if (!awsKubeConfigResult.kubeconfig) {
    throw new Error("no reply or resource for aws kube config");
  }
  const kubeconfigPath = awsKubeConfigResult.kubeconfig;
  if (!kubeconfigPath) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "No kubernetesCluster attached!",
    };

    return {
      failure: {
        resource: {
          state,
          health: ResourceHealth.Error,
          status: ResourceStatus.Failed,
        },
      },
    };
  }

  return { kubeconfigPath, awsEnv };
}

export async function helmCommandPrepare(
  request: AwsInputRequest,
  kubeconfigPath: string,
): Promise<string[]> {
  const tempdir = await fs.mkdtemp(path.join(os.tmpdir(), "helmconfig-"));
  const helmConfigPath = path.join(tempdir, "config");
  const helmCachePath = path.join(tempdir, "cache");
  const helmRegistryConfigPath = path.join(helmConfigPath, "registry.json");
  const helmRepositoryCachePath = path.join(helmCachePath, "repository");
  const helmRepositoryConfigPath = path.join(
    helmConfigPath,
    "repositories.yaml",
  );

  const helmArgs = [
    "--kubeconfig",
    kubeconfigPath,
    "--registry-config",
    helmRegistryConfigPath,
    "--repository-config",
    helmRepositoryConfigPath,
    "--repository-cache",
    helmRepositoryCachePath,
  ];

  return helmArgs;
}

export function helmCommandAuthPrepare(request: AwsInputRequest): string[] {
  const helmArgs = [];
  const helmRepoCredential = findEntityByType(
    request.predecessors,
    "helmRepoCredential",
  );
  if (helmRepoCredential) {
    helmArgs.push("--username");
    helmArgs.push(helmRepoCredential.properties.__baseline.decrypted.username);
    helmArgs.push("--password");
    helmArgs.push(helmRepoCredential.properties.__baseline.decrypted.password);
  }

  if (request.entity.properties.__baseline.insecureSkipTlsVerify) {
    helmArgs.push("--insecure-skip-tls-verify");
  }
  return helmArgs;
}

interface HelmRepoAddResponse {
  helmRepoAdd: SiExecResult;
}

type HelmRepoAddResult = XOR<HelmFailureResult, HelmRepoAddResponse>;

export async function helmRepoAdd(
  request: AwsInputRequest,
  event: Event,
  kubeconfigPath: string,
  awsEnv: AwsCliEnv,
  name: string,
  url: string,
  helmArgs?: string[],
): Promise<HelmRepoAddResult> {
  if (!helmArgs) {
    helmArgs = await helmCommandPrepare(request, kubeconfigPath);
  }

  helmArgs.push("repo", "add", name, url);

  const helmRepoAdd = await siExec(event, "helm", helmArgs, {
    env: awsEnv,
    reject: false,
  });

  if (helmRepoAdd.failed) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "helm repo add failed",
      errorOutput: helmRepoAdd.stderr,
    };

    return {
      failure: {
        resource: {
          state,
          health: ResourceHealth.Error,
          status: ResourceStatus.Failed,
        },
      },
    };
  }
  return { helmRepoAdd };
}
