import { OpSource } from "si-entity/dist/siEntity";
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

import _ from "lodash";
import { RunCommandCallbacks } from "../controllers/runCommand";
import {
  awsAccessKeys,
  awsKubeConfigPath,
  writeKubernetesYaml,
} from "../support";

function inferProperties(
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

export const checkQualifications: CheckQualificationCallbacks = {
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
          [kubeYaml.path, "--ignore-missing-schemas"],
          {
            reject: false,
          },
        );
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

export const runCommands: RunCommandCallbacks = {
  apply: async function (ctx, req, ws) {
    const kubeConfigDir = await awsKubeConfigPath(req);
    const code = req.selection.entity.getCode(req.system.id);
    if (code) {
      const kubeYaml = await writeKubernetesYaml(
        req.selection.entity.getCode(req.system.id),
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
  },
};

export default { inferProperties, checkQualifications, runCommands };
