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
  console.log(`syncing kubernetes`);
  console.dir(request, { depth: Infinity });

  const kubernetesCluster = _.find(request.predecessors, [
    "entity.objectType",
    "kubernetesCluster",
  ]);
  let currentContext = undefined;
  if (kubernetesCluster?.resource.state.data) {
    currentContext = kubernetesCluster.resource.state.data["current-context"];
  }

  if (kubernetesCluster && currentContext) {
    console.log(request.entity.properties.__baseline["kubernetesObjectYaml"]);
    const kubectlApply = await execa(
      "kubectl",
      [
        "apply",
        "-o",
        "json",
        "--context",
        currentContext,
        "--dry-run=server",
        "-f",
        "-",
      ],
      {
        input: request.entity.properties.__baseline["kubernetesObjectYaml"],
      },
    );
    if (kubectlApply.failed) {
      const reply: SyncResourceReply = {
        resource: {
          state: {
            data: request.resource.state?.data,
            errorMsg: "kubectl apply failed",
            errorOutput: kubectlApply.stderr,
          },
          health: ResourceHealth.Ok,
          status: ResourceStatus.Created,
        },
      };
      return reply;
    } else {
      const kubectlApplyJson = JSON.parse(kubectlApply.stdout);
      if (request.entity.objectType == "kubernetesSecret") {
        _.set(kubectlApplyJson, ["data"], "redacted");
        _.set(kubectlApplyJson, ["metadata", "annotations"], "redacted");
      }
      const reply: SyncResourceReply = {
        resource: {
          state: {
            data: kubectlApplyJson,
          },
          health: ResourceHealth.Ok,
          status: ResourceStatus.Created,
        },
      };
      return reply;
    }
  } else {
    const reply: SyncResourceReply = {
      resource: {
        state: {
          data: request.resource.state?.data,
          errorMsg: "No kubernetesCluster attached!",
        },
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
    return reply;
  }
}

export async function kubernetesApply(
  request: ActionRequest,
): Promise<ActionReply> {
  const actions: ActionReply["actions"] = [];
  console.log(`applying kubernetes`);
  console.dir(request, { depth: Infinity });
  const kubernetesCluster = _.find(request.predecessors, [
    "entity.objectType",
    "kubernetesCluster",
  ]);
  let currentContext = undefined;
  if (kubernetesCluster?.resource.state.data) {
    currentContext = kubernetesCluster.resource.state.data["current-context"];
  }

  if (kubernetesCluster && currentContext) {
    const applyArgs = [
      "apply",
      "-o",
      "json",
      "--context",
      currentContext,
      "-f",
      "-",
    ];
    if (request.hypothetical) {
      applyArgs.push("--dry-run=server");
    }

    const kubectlApply = await execa("kubectl", applyArgs, {
      input: request.entity.properties.__baseline["kubernetesObjectYaml"],
    });
    if (kubectlApply.failed) {
      const reply: ActionReply = {
        resource: {
          state: {
            data: request.resource.state?.data,
            errorMsg: "kubectl apply failed",
            errorOutput: kubectlApply.stderr,
          },
          health: ResourceHealth.Ok,
          status: ResourceStatus.Created,
        },
        actions,
      };
      return reply;
    } else {
      const kubectlApplyJson = JSON.parse(kubectlApply.stdout);
      if (request.entity.objectType == "kubernetesSecret") {
        _.set(kubectlApplyJson, ["data"], "redacted");
        _.set(kubectlApplyJson, ["metadata", "annotations"], "redacted");
      }
      const reply: ActionReply = {
        resource: {
          state: {
            data: kubectlApplyJson,
          },
          health: ResourceHealth.Ok,
          status: ResourceStatus.Created,
        },
        actions,
      };
      return reply;
    }
  } else {
    const reply: ActionReply = {
      resource: {
        state: {
          data: request.resource.state?.data,
          errorMsg: "No kubernetesCluster attached!",
        },
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
      actions,
    };
    return reply;
  }
}
