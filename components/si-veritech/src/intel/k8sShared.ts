import { OpSource, SiEntity } from "si-entity/dist/siEntity";
import { Qualification } from "si-registry";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import Debug from "debug";
const debug = Debug("veritech:controllers:intel:k8sNamespace");
import {
  CheckQualificationsItem,
  CheckQualificationsRequest,
} from "../controllers/checkQualifications";
import { SiCtx } from "../siCtx";

import { RunCommandCallbacks } from "../controllers/runCommand";
import {
  awsKubeConfigPath,
  azureKubeConfigPath,
  findEntityByType,
  TempFile,
  TempDir,
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
    cluster: SiEntity,
  ) => Promise<void>,
): Promise<void> {
  const code = req.entity.getCode(req.system.id);
  if (code) {
    const kubeYaml = await writeKubernetesYaml(
      req.entity.getCode(req.system.id),
    );
    const awsEksCluster = findEntityByType(req, "awsEksCluster");
    if (awsEksCluster) {
      const kubeConfigDir = await awsKubeConfigPath(req);
      await callback(kubeYaml, kubeConfigDir, awsEksCluster);
    }
    const azureAksCluster = findEntityByType(req, "azureAksCluster");
    if (azureAksCluster) {
      const kubeConfigDir = await azureKubeConfigPath(req);
      await callback(kubeYaml, kubeConfigDir, azureAksCluster);
    }
  }
}

export const baseRunCommands: RunCommandCallbacks = {
  apply: async function (ctx, req, ws) {
    const awsEksCluster = findEntityByType(req, "awsEksCluster");
    if (awsEksCluster) {
      const kubeConfigDir = await awsKubeConfigPath(req);
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
            "-o",
            "json",
            "--kubeconfig",
            `${kubeConfigDir.path}/config`,
            "-f",
            kubeYaml.path,
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
            "-o",
            "json",
            "--kubeconfig",
            `${kubeConfigDir.path}/config`,
            "-f",
            kubeYaml.path,
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
  const awsEksCluster = findEntityByType(req, "awsEksCluster");

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
    async (_kubeYaml, kubeConfigDir, kubeCluster) => {
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
        { reject: false },
      );
      if (result.exitCode != 0) {
        subResource.state = "unknown";
        subResource.health = "error";
        subResource.internalStatus = ResourceInternalStatus.Failed;
        subResource.internalHealth = ResourceInternalHealth.Error;
        subResource.error = result.all;
        debug("you failed!");
        debug(result.all);
      } else {
        subResource.state = "ok";
        subResource.health = "ok";
        subResource.internalStatus = ResourceInternalStatus.Created;
        subResource.internalHealth = ResourceInternalHealth.Ok;
        subResource.data = JSON.parse(result.stdout);
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
