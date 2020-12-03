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

const intelligence = (registry.get("helmRelease") as EntityObject).intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  const name: string = req.entity.name;
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

  const chart = findEntityByType(request.predecessors, "helmChart");
  helmArgs.push(
    "install",
    request.entity.properties.__baseline.name,
    chart?.properties.__baseline.name,
  );

  const helmAuthArgs = helmCommandAuthPrepare(request);
  helmArgs = helmArgs.concat(helmAuthArgs);

  helmArgs.push("--dry-run");

  if (chart?.properties.__baseline.version) {
    helmArgs.push("--version", chart.properties.__baseline.version);
  }

  const options: { flag: string; if: boolean; value?: string }[] = [
    {
      flag: "--description",
      if: !!request.entity.properties.__baseline.description,
      value: request.entity.properties.__baseline.description,
    },
    {
      flag: "--atomic",
      if: !!request.entity.properties.__baseline.atomic,
    },
    {
      flag: "--no-hooks",
      if: !!request.entity.properties.__baseline.noHooks,
    },
    {
      flag: "--render-subchart-notes",
      if: !!request.entity.properties.__baseline.renderSubchartNotes,
    },
    {
      flag: "--skip-crds",
      if: !!request.entity.properties.__baseline.skipCrds,
    },
    {
      flag: "--timeout",
      if: !!request.entity.properties.__baseline.timeout,
      value: request.entity.properties.__baseline.timeout,
    },
  ];
  for (const option of options) {
    if (option.if) {
      helmArgs.push(option.flag);
      if (option.hasOwnProperty("value")) {
        helmArgs.push(`${option.value}`);
      }
    }
  }

  const helmRepoInstall = await siExec(event, "helm", helmArgs, {
    env: awsEnv,
    reject: false,
  });

  if (helmRepoInstall.failed) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "helm repo install failed",
      errorOutput: helmRepoInstall.stderr,
    };

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  } else {
    return {
      resource: {
        state: {
          data: {
            output: `${helmRepoInstall.all}`,
          },
        },
        health: ResourceHealth.Ok,
        status: ResourceStatus.Created,
      },
    };
  }
};

intelligence.actions = {
  async apply(request: ActionRequest, event: Event): Promise<ActionReply> {
    const reply = { actions: [] };
    const helmKubeConfigResult = await helmKubeConfig(request, event);
    if (helmKubeConfigResult.failure) {
      return { ...reply, ...helmKubeConfigResult.failure };
    }
    const awsEnv = helmKubeConfigResult.awsEnv;
    const kubeconfigPath = helmKubeConfigResult.kubeconfigPath;

    const repo = findEntityByType(request.predecessors, "helmRepo");

    const helmArgs = await helmCommandPrepare(request, kubeconfigPath);

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
      return { ...reply, ...helmRepoAddResult.failure };
    }

    helmArgs.push(
      "list",
      "-f",
      `^${request.entity.properties.__baseline.name}$`,
      "-q",
    );

    const helmList = await siExec(event, "helm", helmArgs, {
      env: awsEnv,
      reject: false,
    });

    if (helmList.failed) {
      const state = {
        data: request.resource.state?.data,
        errorMsg: "helm list failed",
        errorOutput: helmList.stderr,
      };

      return {
        ...reply,
        resource: {
          state,
          health: ResourceHealth.Error,
          status: ResourceStatus.Failed,
        },
      };
    }
    if (helmList.all == request.entity.properties.__baseline.name) {
      if (intelligence.actions?.upgrade) {
        return await intelligence.actions.upgrade(request, event);
      }
    } else {
      if (intelligence.actions?.install) {
        return await intelligence.actions.install(request, event);
      }
    }
    const state = {
      data: request.resource.state?.data,
      errorMsg: "cannot decide what to do with helm!! bug",
      errorOutput: helmList.stderr,
    };

    return {
      ...reply,
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  },

  async upgrade(request: ActionRequest, event: Event): Promise<ActionReply> {
    const reply = { actions: [] };
    const helmKubeConfigResult = await helmKubeConfig(request, event);
    if (helmKubeConfigResult.failure) {
      return { ...reply, ...helmKubeConfigResult.failure };
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
      return { ...reply, ...helmRepoAddResult.failure };
    }

    const chart = findEntityByType(request.predecessors, "helmChart");
    helmArgs.push(
      "upgrade",
      request.entity.properties.__baseline.name,
      chart?.properties.__baseline.name,
    );

    const helmAuthArgs = helmCommandAuthPrepare(request);
    helmArgs = helmArgs.concat(helmAuthArgs);

    if (chart?.properties.__baseline.version) {
      helmArgs.push("--version", chart.properties.__baseline.version);
    }

    const options: { flag: string; if: boolean; value?: string }[] = [
      {
        flag: "--dry-run",
        if: request.hypothetical,
      },
      {
        flag: "--description",
        if: !!request.entity.properties.__baseline.description,
        value: request.entity.properties.__baseline.description,
      },
      {
        flag: "--atomic",
        if: !!request.entity.properties.__baseline.atomic,
      },
      {
        flag: "--no-hooks",
        if: !!request.entity.properties.__baseline.noHooks,
      },
      {
        flag: "--render-subchart-notes",
        if: !!request.entity.properties.__baseline.renderSubchartNotes,
      },
      {
        flag: "--skip-crds",
        if: !!request.entity.properties.__baseline.skipCrds,
      },
      {
        flag: "--wait",
        if: !!request.entity.properties.__baseline.wait,
      },
      {
        flag: "--timeout",
        if: !!request.entity.properties.__baseline.timeout,
        value: request.entity.properties.__baseline.timeout,
      },
    ];
    for (const option of options) {
      if (option.if) {
        helmArgs.push(option.flag);
        if (option.hasOwnProperty("value")) {
          helmArgs.push(`${option.value}`);
        }
      }
    }

    const helmRepoInstall = await siExec(event, "helm", helmArgs, {
      env: awsEnv,
      reject: false,
    });

    if (helmRepoInstall.failed) {
      const state = {
        data: request.resource.state?.data,
        errorMsg: "helm repo install failed",
        errorOutput: helmRepoInstall.stderr,
      };

      return {
        ...reply,
        resource: {
          state,
          health: ResourceHealth.Error,
          status: ResourceStatus.Failed,
        },
      };
    } else {
      return {
        ...reply,
        resource: {
          state: {
            data: {
              output: `${helmRepoInstall.all}`,
            },
          },
          health: ResourceHealth.Ok,
          status: ResourceStatus.Created,
        },
      };
    }
  },

  async install(request: ActionRequest, event: Event): Promise<ActionReply> {
    const reply = { actions: [] };
    const helmKubeConfigResult = await helmKubeConfig(request, event);
    if (helmKubeConfigResult.failure) {
      return { ...reply, ...helmKubeConfigResult.failure };
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
      return { ...reply, ...helmRepoAddResult.failure };
    }

    const chart = findEntityByType(request.predecessors, "helmChart");
    helmArgs.push(
      "install",
      request.entity.properties.__baseline.name,
      chart?.properties.__baseline.name,
    );

    const helmAuthArgs = helmCommandAuthPrepare(request);
    helmArgs = helmArgs.concat(helmAuthArgs);

    if (chart?.properties.__baseline.version) {
      helmArgs.push("--version", chart.properties.__baseline.version);
    }

    const options: { flag: string; if: boolean; value?: string }[] = [
      {
        flag: "--dry-run",
        if: request.hypothetical,
      },
      {
        flag: "--description",
        if: !!request.entity.properties.__baseline.description,
        value: request.entity.properties.__baseline.description,
      },
      {
        flag: "--atomic",
        if: !!request.entity.properties.__baseline.atomic,
      },
      {
        flag: "--no-hooks",
        if: !!request.entity.properties.__baseline.noHooks,
      },
      {
        flag: "--render-subchart-notes",
        if: !!request.entity.properties.__baseline.renderSubchartNotes,
      },
      {
        flag: "--skip-crds",
        if: !!request.entity.properties.__baseline.skipCrds,
      },
      {
        flag: "--wait",
        if: !!request.entity.properties.__baseline.wait,
      },
      {
        flag: "--timeout",
        if: !!request.entity.properties.__baseline.timeout,
        value: request.entity.properties.__baseline.timeout,
      },
    ];
    for (const option of options) {
      if (option.if) {
        helmArgs.push(option.flag);
        if (option.hasOwnProperty("value")) {
          helmArgs.push(`${option.value}`);
        }
      }
    }

    const helmRepoInstall = await siExec(event, "helm", helmArgs, {
      env: awsEnv,
      reject: false,
    });

    if (helmRepoInstall.failed) {
      const state = {
        data: request.resource.state?.data,
        errorMsg: "helm repo install failed",
        errorOutput: helmRepoInstall.stderr,
      };

      return {
        ...reply,
        resource: {
          state,
          health: ResourceHealth.Error,
          status: ResourceStatus.Failed,
        },
      };
    } else {
      return {
        ...reply,
        resource: {
          state: {
            data: {
              output: `${helmRepoInstall.all}`,
            },
          },
          health: ResourceHealth.Ok,
          status: ResourceStatus.Created,
        },
      };
    }
  },
};
