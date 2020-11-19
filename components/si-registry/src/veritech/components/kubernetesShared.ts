import {
  Entity,
  CalculatePropertiesResult,
  SyncResourceRequest,
  SyncResourceReply,
  ResourceHealth,
  ResourceStatus,
  ActionRequest,
  ActionReply,
} from "../intelligence";
import _ from "lodash";
import execa from "execa";
import { awsCredential, awsKubeConfig, AwsCliEnv } from "./awsShared";

// A canonical apply/install order, thanks to the Helm project. This is the
// order of applying Kubernetes object when running a `helm install` command.
//
// Helm source reference: https://git.io/Jk0Y5
export const kubernetesApplyOrder = [
  "kubernetesNamespace",
  "kubernetesNetworkPolicy",
  "kubernetesResourceQuota",
  "kubernetesLimitRange",
  "kubernetesPodSecurityPolicy",
  "kubernetesPodDisruptionBudget",
  "kubernetesServiceAccount",
  "kubernetesSecret",
  "kubernetesSecretList",
  "kubernetesConfigMap",
  "kubernetesStorageClass",
  "kubernetesPersistentVolume",
  "kubernetesPersistentVolumeClaim",
  "kubernetesCustomResourceDefinition",
  "kubernetesClusterRole",
  "kubernetesClusterRoleList",
  "kubernetesClusterRoleBinding",
  "kubernetesClusterRoleBindingList",
  "kubernetesRole",
  "kubernetesRoleList",
  "kubernetesRoleBinding",
  "kubernetesRoleBindingList",
  "kubernetesService",
  "kubernetesDaemonSet",
  "kubernetesPod",
  "kubernetesReplicationController",
  "kubernetesReplicaSet",
  "kubernetesDeployment",
  "kubernetesHorizontalPodAutoscaler",
  "kubernetesStatefulSet",
  "kubernetesJob",
  "kubernetesCronJob",
  "kubernetesIngress",
  "kubernetesAPIService",
];

export function kubernetesNamespaceProperties(
  result: CalculatePropertiesResult,
  namespace: Entity,
): CalculatePropertiesResult {
  if (namespace.properties.__baseline.kubernetesObject?.metadata?.name) {
    _.set(
      result.inferredProperties,
      ["__baseline", "kubernetesObject", "metadata", "namespace"],
      namespace.properties.__baseline.kubernetesObject.metadata.name,
    );
  }
  return result;
}

export async function kubernetesSync(
  request: SyncResourceRequest,
): Promise<SyncResourceReply> {
  const awsCredResult = awsCredential(request);
  if (awsCredResult.syncResourceReply) {
    return awsCredResult.syncResourceReply;
  } else if (!awsCredResult.awsCliEnv) {
    throw new Error("aws cli function didn't return an environment");
  }
  const awsEnv: AwsCliEnv = awsCredResult.awsCliEnv;
  const awsKubeConfigResult = await awsKubeConfig(request, awsEnv);
  if (awsKubeConfigResult.syncResourceReply) {
    return awsKubeConfigResult.syncResourceReply;
  } else if (!awsKubeConfigResult.kubeconfig) {
    throw new Error("no reply or resource for aws kube config");
  }
  const kubeconfigPath = awsKubeConfigResult.kubeconfig;

  // If no valid kubeconfig, early return
  if (!kubeconfigPath) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "No kubernetesCluster attached!",
    };

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  }

  const kubectlArgs = [
    "apply",
    "-o",
    "json",
    "--kubeconfig",
    kubeconfigPath,
    "--dry-run=server",
    "-f",
    "-",
  ];
  console.log(`running command; cmd="kubectl ${kubectlArgs.join(" ")}"`);
  const kubectlApply = await execa("kubectl", kubectlArgs, {
    reject: false,
    input: request.entity.properties.__baseline["kubernetesObjectYaml"],
    env: awsEnv,
  });

  // If kubectl apply failed, early return
  if (kubectlApply.failed) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "kubectl apply command failed",
      errorOutput: kubectlApply.stderr,
    };

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  }

  const kubectlApplyJson = JSON.parse(kubectlApply.stdout);
  if (request.entity.objectType == "kubernetesSecret") {
    _.set(kubectlApplyJson, ["data"], "redacted");
    _.set(kubectlApplyJson, ["metadata", "annotations"], "redacted");
  }

  return {
    resource: {
      state: {
        data: kubectlApplyJson,
      },
      health: ResourceHealth.Ok,
      status: ResourceStatus.Created,
    },
  };
}

export async function kubernetesApply(
  request: ActionRequest,
): Promise<ActionReply> {
  const actions: ActionReply["actions"] = [];
  const awsCredResult = awsCredential(request);
  if (awsCredResult.syncResourceReply) {
    return { resource: awsCredResult.syncResourceReply.resource, actions };
  } else if (!awsCredResult.awsCliEnv) {
    throw new Error("aws cli function didn't return an environment");
  }
  const awsEnv: AwsCliEnv = awsCredResult.awsCliEnv;
  const awsKubeConfigResult = await awsKubeConfig(request, awsEnv);
  if (awsKubeConfigResult.syncResourceReply) {
    return {
      resource: awsKubeConfigResult.syncResourceReply.resource,
      actions,
    };
  } else if (!awsKubeConfigResult.kubeconfig) {
    throw new Error("no reply or resource for aws kube config");
  }
  const kubeconfigPath = awsKubeConfigResult.kubeconfig;

  // If no valid kubeconfig, early return
  if (!kubeconfigPath) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "No awsEks node attached!",
    };

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
      actions,
    };
  }

  const kubectlArgs = [
    "apply",
    "-o",
    "json",
    "--kubeconfig",
    kubeconfigPath,
    "-f",
    "-",
  ];
  if (request.hypothetical) {
    kubectlArgs.push("--dry-run=server");
  }
  console.log(`running command; cmd="kubectl ${kubectlArgs.join(" ")}"`);
  const kubectlApply = await execa("kubectl", kubectlArgs, {
    reject: false,
    input: request.entity.properties.__baseline["kubernetesObjectYaml"],
    env: awsEnv,
  });

  // If kubectl apply failed, early return
  if (kubectlApply.failed) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "kubectl apply command failed",
      errorOutput: kubectlApply.stderr,
    };

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
      actions,
    };
  }

  const kubectlApplyJson = JSON.parse(kubectlApply.stdout);
  if (request.entity.objectType == "kubernetesSecret") {
    _.set(kubectlApplyJson, ["data"], "redacted");
    _.set(kubectlApplyJson, ["metadata", "annotations"], "redacted");
  }

  return {
    resource: {
      state: {
        data: kubectlApplyJson,
      },
      health: ResourceHealth.Ok,
      status: ResourceStatus.Created,
    },
    actions,
  };
}
