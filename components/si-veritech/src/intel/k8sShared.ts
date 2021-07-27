import Debug from "debug";
const debug = Debug("veritech:controllers:intel:k8sShared");

import { OpSource, SiEntity } from "si-entity/dist/siEntity";
import { Qualification } from "si-registry";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import {
  CheckQualificationsItem,
  CheckQualificationsRequest,
} from "../controllers/checkQualifications";
import { SiCtx } from "../siCtx";

import { RunCommandCallbacks } from "../controllers/runCommand";
import {
  awsAccessKeysEnvironment,
  awsKubeConfigPath,
  azureKubeConfigPath,
  findEntityByType,
  writeKubernetesYaml,
} from "../support";
import {
  CommandProtocolFinish,
  SyncResourceRequest,
} from "../controllers/syncResource";
import WebSocket from "ws";
import {
  ResourceInternalHealth,
  ResourceInternalStatus,
  SubResource,
} from "si-entity";

const NS_TYPE = "k8sNamespace";

export function baseInferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const entity = request.entity;

  entity.set({
    source: OpSource.Inferred,
    system: "baseline",
    path: ["metadata", "name"],
    value: request.entity.name,
  });

  return { entity };
}

export type CheckQualificationCallback = (
  ctx: typeof SiCtx,
  qualification: Qualification,
  request: CheckQualificationsRequest,
) => Promise<CheckQualificationsItem>;

export interface CheckQualificationCallbacks {
  [qualificationName: string]: CheckQualificationCallback;
}

export const baseCheckQualifications: CheckQualificationCallbacks = {
  async kubeval(ctx, q, r) {
    const code = r.entity.getCode(r.systemId);
    let qualified = false;
    let output = "";
    if (code) {
      const kubeYaml = await writeKubernetesYaml(r.entity.getCode(r.systemId));

      qualified = false;
      let kubeval;
      try {
        kubeval = await ctx.exec(
          "kubeval",
          [kubeYaml.path, "--ignore-missing-schemas", "-o", "json", "--quiet"],
          {
            reject: false,
          },
        );
        console.log(kubeval);
        if (kubeval.exitCode == 0) {
          qualified = true;
          output = kubeval.all;
        } else {
          qualified = false;
          output = kubeval.all;
        }
      } catch (e) {
        output = `${e}`;
        debug(e);
      } finally {
        await kubeYaml.cleanup();
      }
    } else {
      qualified = false;
      output = JSON.stringify(r.entity);
    }
    return {
      name: q.name,
      qualified,
      output,
    };
  },
};

export async function forEachCluster(
  ctx: any,
  req: any,
  ws: any,
  callback: (
    kubeYaml: any,
    kubeConfigDir: any,
    env: Record<string, string>,
    cluster: SiEntity,
  ) => Promise<void>,
): Promise<void> {
  const code = req.entity.getCode(req.system.id);
  if (code) {
    const kubeYaml = await writeKubernetesYaml(
      req.entity.getCode(req.system.id),
    );
    const execEnv = awsAccessKeysEnvironment(req);
    const awsEksCluster = findEntityByType(req, "awsEksCluster");
    if (awsEksCluster) {
      const kubeConfigDir = await awsKubeConfigPath(req);
      await callback(kubeYaml, kubeConfigDir, execEnv, awsEksCluster);
    }
    const azureAksCluster = findEntityByType(req, "azureAksCluster");
    if (azureAksCluster) {
      const kubeConfigDir = await azureKubeConfigPath(req);
      await callback(kubeYaml, kubeConfigDir, {}, azureAksCluster);
    }
  }
}

export const baseRunCommands: RunCommandCallbacks = {
  apply: async function (ctx, req, ws) {
    const awsEksCluster = findEntityByType(req, "awsEksCluster");
    if (awsEksCluster) {
      const kubeConfigDir = await awsKubeConfigPath(req);
      const awsEnv = awsAccessKeysEnvironment(req);
      const code = req.entity.getCode(req.system.id);
      if (code) {
        const kubeYaml = await writeKubernetesYaml(
          req.entity.getCode(req.system.id),
        );
        const result = await ctx.execStream(
          ws,
          "kubectl",
          [
            "apply",
            "--output=json",
            `--kubeconfig=${kubeConfigDir.path}/config`,
            `--filename=${kubeYaml.path}`,
          ],
          { env: awsEnv, reject: false },
        );
        if (result.exitCode != 0) {
          debug("you failed!");
          debug(result.all);
        } else {
          debug("you worked!");
          debug(result.all);
        }
      } else {
        await ctx.execStream(ws, "echo", ["no code, so no apply!"]);
      }
    }
    const azureAksCluster = findEntityByType(req, "azureAksCluster");
    if (azureAksCluster) {
      const kubeConfigDir = await azureKubeConfigPath(req);
      const code = req.entity.getCode(req.system.id);
      if (code) {
        const kubeYaml = await writeKubernetesYaml(
          req.entity.getCode(req.system.id),
        );
        const result = await ctx.execStream(
          ws,
          "kubectl",
          [
            "apply",
            "--output=json",
            `--kubeconfig=${kubeConfigDir.path}/config`,
            `--filename=${kubeYaml.path}`,
          ],
          { reject: false },
        );
        if (result.exitCode != 0) {
          debug("you failed!");
          debug(result.all);
        } else {
          debug("you worked!");
          debug(result.all);
        }
      } else {
        await ctx.execStream(ws, "echo", ["no code, so no apply!"]);
      }
    }
  },
  delete: async function (ctx, req, ws) {
    debug("deleting!");
    const awsEksCluster = findEntityByType(req, "awsEksCluster");
    if (awsEksCluster) {
      const kubeConfigDir = await awsKubeConfigPath(req);
      const awsEnv = awsAccessKeysEnvironment(req);
      const code = req.entity.getCode(req.system.id);
      if (code) {
        let args = ["delete", `--kubeconfig=${kubeConfigDir.path}/config`];
        if (req.entity.entityType != NS_TYPE) {
          const connectedNamespace = findEntityByType(req, NS_TYPE);
          if (connectedNamespace) {
            args.push(
              `--namespace=${k8sName(connectedNamespace, req.system.id)}`,
            );
          }
        }
        args = args.concat([
          k8sObjTypeFromEntityType(req.entity.entityType),
          k8sName(req.entity, req.system.id),
        ]);

        const result = await ctx.execStream(ws, "kubectl", args, {
          env: awsEnv,
          reject: false,
        });
        if (result.exitCode != 0) {
          debug("you failed!");
          debug(result.all);
        } else {
          debug("you worked!");
          debug(result.all);
        }
      } else {
        await ctx.execStream(ws, "echo", ["no code, so no delete!"]);
      }
    }
    const azureAksCluster = findEntityByType(req, "azureAksCluster");
    if (azureAksCluster) {
      const kubeConfigDir = await azureKubeConfigPath(req);
      const code = req.entity.getCode(req.system.id);
      if (code) {
        let args = ["delete", `--kubeconfig=${kubeConfigDir.path}/config`];
        if (req.entity.entityType != NS_TYPE) {
          const connectedNamespace = findEntityByType(req, NS_TYPE);
          if (connectedNamespace) {
            args.push(
              `--namespace=${k8sName(connectedNamespace, req.system.id)}`,
            );
          }
        }
        args = args.concat([
          k8sObjTypeFromEntityType(req.entity.entityType),
          k8sName(req.entity, req.system.id),
        ]);

        const result = await ctx.execStream(ws, "kubectl", args, {
          reject: false,
        });
        if (result.exitCode != 0) {
          debug("you failed!");
          debug(result.all);
        } else {
          debug("you worked!");
          debug(result.all);
        }
      } else {
        await ctx.execStream(ws, "echo", ["no code, so no apply!"]);
      }
    }
  },
};

export async function baseSyncResource(
  ctx: typeof SiCtx,
  req: SyncResourceRequest,
  ws: WebSocket,
): Promise<CommandProtocolFinish["finish"]> {
  const response: CommandProtocolFinish["finish"] = {
    data: req.resource.data,
    state: req.resource.state,
    health: req.resource.health,
    internalStatus: req.resource.internalStatus,
    internalHealth: req.resource.internalHealth,
    subResources: req.resource.subResources,
  };

  const nameSpace = findEntityByType(req, "k8sNamespace");

  const defaultArgs = ["get", "-o", "json"];
  if (nameSpace) {
    defaultArgs.push("-n");
    defaultArgs.push(
      nameSpace.getProperty({
        system: req.system.id,
        path: ["metadata", "name"],
      }),
    );
  }
  await forEachCluster(
    ctx,
    req,
    ws,
    async (_kubeYaml, kubeConfigDir, execEnv, kubeCluster) => {
      let subResource: SubResource;
      if (response.subResources[kubeCluster.id]) {
        subResource = response.subResources[kubeCluster.id];
      } else {
        subResource = {
          unixTimestamp: req.resource.unixTimestamp,
          timestamp: req.resource.timestamp,
          data: {},
          state: "unknown",
          health: "unknown",
          internalStatus: ResourceInternalStatus.Pending,
          internalHealth: ResourceInternalHealth.Unknown,
        };
      }
      const kind = req.entity.getProperty({
        system: req.system.id,
        path: ["kind"],
      }) as string;
      const name = req.entity.getProperty({
        system: req.system.id,
        path: ["metadata", "name"],
      }) as string;
      const result = await ctx.exec(
        "kubectl",
        [
          ...defaultArgs,
          "--kubeconfig",
          `${kubeConfigDir.path}/config`,
          kind,
          name,
        ],
        { env: execEnv, reject: false },
      );
      if (result.exitCode != 0) {
        subResource.state = "unknown";
        subResource.health = "error";
        subResource.internalStatus = ResourceInternalStatus.Failed;
        subResource.internalHealth = ResourceInternalHealth.Error;
        subResource.error = result.all;
        subResource.data["clusterName"] = kubeCluster.name;
        subResource.data["clusterType"] = kubeCluster.entityType;
        debug("you failed!");
        debug(result.all);
      } else {
        subResource.state = "ok";
        subResource.health = "ok";
        subResource.internalStatus = ResourceInternalStatus.Created;
        subResource.internalHealth = ResourceInternalHealth.Ok;
        subResource.data = JSON.parse(result.stdout);
        subResource.data["clusterName"] = kubeCluster.name;
        subResource.data["clusterType"] = kubeCluster.entityType;
        subResource.error = null;
        debug("you worked!");
        debug(result.all);
      }
      response.subResources[kubeCluster.id] = subResource;
    },
  );

  if (Object.keys(response.subResources).length > 0) {
    let internalHealth = ResourceInternalHealth.Ok;
    let internalStatus = ResourceInternalStatus.Created;
    let health = "ok";
    let state = "ok";
    for (const cluster of Object.values(response.subResources)) {
      if (cluster.internalHealth != ResourceInternalHealth.Ok) {
        internalHealth = ResourceInternalHealth.Error;
        health = "error";
      }
      if (cluster.internalStatus != ResourceInternalStatus.Created) {
        internalStatus = cluster.internalStatus;
        state = cluster.state;
      }
    }
    response.internalHealth = internalHealth;
    response.internalStatus = internalStatus;
    response.health = health;
    response.state = state;
    response.data = response.subResources;
    response.error = null;
  } else {
    response.internalHealth = ResourceInternalHealth.Unknown;
    response.internalStatus = ResourceInternalStatus.Failed;
    response.health = "unknown";
    response.state = "unknown";
    response.error = "no cluster attached";
  }
  return response;
}

function k8sObjTypeFromEntityType(entityType: string): string {
  switch (entityType) {
    case "k8sNamespace":
      return "namespaces";
    case "k8sNetworkPolicy":
      return "networkpolicies.networking.k8s.io";
    case "k8sResourceQuota":
      return "resourcequotas";
    case "k8sLimitRange":
      return "limitranges";
    case "k8sPodSecurityPolicy":
      return "podsecuritypolicies.policy";
    case "k8sPodDisruptionBudget":
      return "poddisruptionbudgets.policy";
    case "k8sSecret":
      return "secrets";
    case "k8sConfigMap":
      return "configmaps";
    case "k8sStorageClass":
      return "storageclasses.storage.k8s.io";
    case "k8sPersistentVolume":
      return "persistentvolumes";
    case "k8sPersistentVolumeClaim":
      return "persistentvolumeclaims";
    case "k8sServiceAccount":
      return "serviceaccounts";
    case "k8sCustomResourceDefinition":
      return "customresourcedefinitions.apiextensions.k8s.io";
    case "k8sClusterRole":
      return "clusterroles.rbac.authorization.k8s.io";
    case "k8sClusterRoleBinding":
      return "clusterrolebindings.rbac.authorization.k8s.io";
    case "k8sRole":
      return "roles.rbac.authorization.k8s.io";
    case "k8sRoleBinding":
      return "rolebindings.rbac.authorization.k8s.io";
    case "k8sService":
      return "services";
    case "k8sDaemonSet":
      return "daemonsets.apps";
    case "k8sPod":
      return "pods";
    case "k8sReplicationController":
      return "replicationcontrollers";
    case "k8sReplicaSet":
      return "replicasets.apps";
    case "k8sDeployment":
      return "deployments.apps";
    case "k8sHorizontalPodAutoscaler":
      return "horizontalpodautoscalers.autoscaling";
    case "k8sStatefulSet":
      return "statefulsets.apps";
    case "k8sJob":
      return "jobs.batch";
    case "k8sCronJob":
      return "cronjobs.batch";
    case "k8sIngress":
      return "ingresses.networking.k8s.io";
    case "k8sAPIService":
      return "apiservices.apiregistration.k8s.io";

    case "k8sClusterRoleList":
    case "k8sClusterRoleBindingList":
    case "k8sRoleList":
    case "k8sRoleBindingList":
    default:
      throw Error(`Unknown k8s type name for entityType: ${entityType}`);
  }
}

function k8sName(entity: SiEntity, systemId: string): string {
  return entity.getProperty({
    system: systemId,
    path: ["metadata", "name"],
  });
}
