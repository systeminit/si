import { registry } from "@/registry";
import { EntityObject } from "@/systemComponent";
import {
  ActionRequest,
  ActionReply,
  SyncResourceRequest,
  SyncResourceReply,
  CalculatePropertiesRequest,
  CalculatePropertiesResult,
  findEntityByType,
  ResourceHealth,
  ResourceStatus,
} from "../../veritech/intelligence";
import { Event } from "../../veritech/eventLog";
import {
  kubernetesNamespaceProperties,
  kubernetesSync,
  kubernetesApply,
} from "./kubernetesShared";
import { awsCredential, awsKubeConfig, AwsCliEnv } from "./awsShared";
import { siExec } from "../siExec";
import { promises as fs } from "fs";
import os from "os";
import path from "path";
import {
  helmCommandPrepare,
  helmKubeConfig,
  helmCommandAuthPrepare,
  helmRepoAdd,
} from "./helmShared";
import _ from "lodash";
import yaml from "yaml";

const intelligence = (registry.get("helmChart") as EntityObject).intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  const helmRepo = findEntityByType(req.predecessors, "helmRepo");
  let name: string;
  if (helmRepo) {
    name = `${helmRepo.properties.__baseline.name}/${req.entity.name}`;
  } else {
    name = req.entity.name;
  }
  const result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {
        name,
      },
    },
  };
  return result;
};

intelligence.syncResource = async function(
  request: SyncResourceRequest,
  event: Event,
): Promise<SyncResourceReply> {
  const helmKubeConfigResult = await helmKubeConfig(request, event);
  if (helmKubeConfigResult.failure) {
    return helmKubeConfigResult.failure;
  }
  const awsEnv = helmKubeConfigResult.awsEnv;
  const kubeconfigPath = helmKubeConfigResult.kubeconfigPath;

  const repo = findEntityByType(request.predecessors, "helmRepo");

  let helmArgs = await helmCommandPrepare(request, kubeconfigPath);

  const helmRepoAddResult = await helmRepoAdd(
    request,
    event,
    kubeconfigPath,
    awsEnv,
    repo?.properties.__baseline.name,
    repo?.properties.__baseline.url,
    _.clone(helmArgs),
  );
  if (helmRepoAddResult.failure) {
    return helmRepoAddResult.failure;
  }

  const chartName: string = request.entity.properties.__baseline.name;
  helmArgs.push("pull");
  helmArgs.push(chartName);

  const helmAuthArgs = helmCommandAuthPrepare(request);
  helmArgs = helmArgs.concat(helmAuthArgs);

  const tempdir = await fs.mkdtemp(path.join(os.tmpdir(), "helmpull-"));
  helmArgs.push("--destination");
  helmArgs.push(tempdir);
  helmArgs.push("--untar");

  if (request.entity.properties.__baseline.version) {
    helmArgs.push("--version", request.entity.properties.__baseline.version);
  }

  const helmRepoPull = await siExec(event, "helm", helmArgs, {
    env: awsEnv,
    reject: false,
  });

  if (helmRepoPull.failed) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "helm repo pull failed",
      errorOutput: helmRepoPull.stderr,
    };

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  } else {
    const chartFullName: string = request.entity.properties.__baseline.name;
    const chartParts = chartFullName.split("/");
    const chartName = chartParts[1];

    const chartYamlRaw = await fs.readFile(
      path.join(tempdir, chartName, "Chart.yaml"),
    );
    const chartYaml = yaml.parse(chartYamlRaw.toString());

    let valuesYaml = {};
    try {
      const valuesYamlRaw = await fs.readFile(
        path.join(tempdir, chartName, "values.yaml"),
      );
      valuesYaml = yaml.parse(valuesYamlRaw.toString());
    } catch {}

    let valuesSchemaJson = {};
    try {
      const valuesSchemaJsonRaw = await fs.readFile(
        path.join(tempdir, chartName, "values.schema.json"),
      );
      valuesSchemaJson = JSON.parse(valuesSchemaJsonRaw.toString());
    } catch (_e) {}

    return {
      resource: {
        state: {
          data: {
            chartYaml,
            valuesYaml,
            valuesSchemaJson,
          },
        },
        health: ResourceHealth.Ok,
        status: ResourceStatus.Created,
      },
    };
  }
};
