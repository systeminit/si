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
import { helmKubeConfig, helmRepoAdd } from "./helmShared";

const intelligence = (registry.get("helmRepo") as EntityObject).intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  const result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {
        name: `${req.entity.name}`,
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
  const helmRepoAddResult = await helmRepoAdd(
    request,
    event,
    kubeconfigPath,
    awsEnv,
    request.entity.properties.__baseline.name,
    request.entity.properties.__baseline.url,
  );

  if (helmRepoAddResult.failure) {
    return helmRepoAddResult.failure;
  }
  return {
    resource: {
      state: {
        data: {
          added: true,
        },
      },
      health: ResourceHealth.Ok,
      status: ResourceStatus.Created,
    },
  };
};
