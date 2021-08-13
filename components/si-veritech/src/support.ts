import tmp from "tmp-promise";
import { DirOptions, FileOptions } from "tmp";
import { promises as fs } from "fs";
import path from "path";
import Debug from "debug";
const debug = Debug("veritech:support");
import _ from "lodash";

import { RunCommandRequest } from "./controllers/runCommand";
import { OpSource, OpType, SiEntity } from "si-entity";
import { SiCtx } from "./siCtx";

export type TempFile = ReturnType<typeof tmp.file>;
export type TempDir = ReturnType<typeof tmp.dir>;
export type Context = Pick<RunCommandRequest, "context" | "system">;

export enum SecretKind {
  DockerHub = "dockerHub",
  AwsAccessKey = "awsAccessKey",
  HelmRepo = "helmRepo",
  AzureServicePrincipal = "azureServicePrincipal",
}

export interface DecryptedSecret {
  id: string;
  name: string;
  objectType: "credential";
  kind: SecretKind;
  message: Record<string, any>;
}

export async function tempDir(options: DirOptions): Promise<TempDir> {
  return tmp.dir(options);
}

export async function writeTempFile(
  content: string,
  options: FileOptions,
): Promise<TempFile> {
  const fileResult = await tmp.file(options);
  try {
    await fs.appendFile(fileResult.path, content);
  } catch (e) {
    debug("we errored writing our file");
    debug({ e, content, options });
    throw e;
  }
  return fileResult;
}

export async function writeKubernetesYaml(content: string): Promise<TempFile> {
  const result = writeTempFile(content, { mode: 0o600, postfix: ".yml" });
  return result;
}

export function findSecret(
  context: Context,
  kind: SecretKind,
): Record<string, any> | null {
  const contextItem = _.find(context.context, (c) => c.secret?.kind == kind);
  if (contextItem && contextItem.secret) {
    return contextItem.secret.message;
  } else {
    return null;
  }
}

export function findEntityByType(
  context: Context,
  entityType: string,
): SiEntity | null {
  const contextItem = _.find(
    context.context,
    (c) => c.entity.entityType == entityType,
  );
  if (contextItem?.entity) {
    return contextItem.entity;
  } else {
    return null;
  }
}

export function findProperty(
  context: Context,
  entityType: string,
  path: string[],
): any {
  const entity = findEntityByType(context, entityType);
  if (entity) {
    const value = entity.getProperty({ path, system: context.system.id });
    if (value) {
      return value;
    } else {
      return null;
    }
  } else {
    return null;
  }
}

export interface AwsAccessKeyEnv {
  AWS_ACCESS_KEY_ID: string;
  AWS_SECRET_ACCESS_KEY: string;
  AWS_DEFAULT_OUTPUT: string;
  [key: string]: string;
}

export function awsAccessKeysEnvironment(context: Context): AwsAccessKeyEnv {
  const response: Partial<AwsAccessKeyEnv> = {
    AWS_DEFAULT_OUTPUT: "json",
  };
  const secret = findSecret(context, SecretKind.AwsAccessKey);
  if (secret) {
    response["AWS_ACCESS_KEY_ID"] = secret["accessKeyId"];
    response["AWS_SECRET_ACCESS_KEY"] = secret["secretAccessKey"];
  } else {
    throw new Error("AWS Access Keys requested, and none found as inputs!");
  }
  return response as AwsAccessKeyEnv;
}

export function awsRegion(context: Context): string {
  const region = findProperty(context, "awsRegion", ["region"]);
  if (region) {
    return region;
  } else {
    throw new Error("AWS Region requested, but none found as inputs!");
  }
}

export function awsEksClusterName(context: Context): string {
  const clusterName = findProperty(context, "awsEksCluster", ["name"]);
  if (clusterName) {
    return clusterName;
  } else {
    throw new Error("AWS Eks Cluster requested, but notn found as inputs!");
  }
}

export interface KubeConfigDir {
  tempDir: TempDir;
  directory: string;
}

export async function awsKubeConfigPath(
  context: Context,
  clusterName?: string,
): Promise<TempDir> {
  const awsEnv = awsAccessKeysEnvironment(context);
  const region = awsRegion(context);
  const defaultArgs = ["--region", region];
  if (!clusterName) {
    clusterName = awsEksClusterName(context);
  }
  const kubeTempDir = await tempDir({});
  const kubeconfigPath = path.join(kubeTempDir.path, "config");
  await SiCtx.exec(
    "aws",
    [
      ...defaultArgs,
      "eks",
      "update-kubeconfig",
      "--name",
      clusterName,
      "--kubeconfig",
      kubeconfigPath,
    ],
    {
      env: awsEnv,
    },
  );
  return kubeTempDir;
}

export async function azureLogin(
  context: Context,
  secret?: Record<string, any>,
): Promise<void> {
  if (!secret) {
    secret = findSecret(context, SecretKind.AzureServicePrincipal);
  }
  if (secret) {
    await SiCtx.exec("az", [
      "login",
      "--service-principal",
      "-u",
      secret["servicePrincipalUri"],
      "-p",
      secret["password"],
      "--tenant",
      secret["tenant"],
    ]);
  } else {
    throw new Error(
      "Azure Service Principal requested, and none found as inputs!",
    );
  }
}

export function azureResourceGroup(context: Context): string {
  const rgName = findProperty(context, "azureResourceGroup", ["name"]);
  if (rgName) {
    return rgName;
  } else {
    throw new Error(
      "Azure Resource Group requested, but none found as inputs!",
    );
  }
}

export function azureAksClusterName(context: Context): string {
  const aksName = findProperty(context, "azureAksCluster", ["name"]);
  if (aksName) {
    return aksName;
  } else {
    throw new Error("Azure AKS Cluster requested, but none found as inputs!");
  }
}

export async function azureKubeConfigPath(
  context: Context,
  clusterName?: string,
): Promise<TempDir> {
  await azureLogin(context);
  const resourceGroup = azureResourceGroup(context);
  if (!clusterName) {
    clusterName = azureAksClusterName(context);
  }
  const kubeTempDir = await tempDir({});
  const kubeconfigPath = path.join(kubeTempDir.path, "config");
  await SiCtx.exec("az", [
    "aks",
    "get-credentials",
    "--resource-group",
    resourceGroup,
    "--name",
    clusterName,
    "--file",
    kubeconfigPath,
  ]);
  return kubeTempDir;
}

export function k8sDiscoverEntity(
  entity: SiEntity,
  data: Record<string, any>,
): void {
  const path: string[] = [];
  const entityName = _.get(data, ["metadata", "name"]);
  //_k8sDiscoverEntityObject(entity, data, path);
}

function _k8sDiscoverEntityArray(
  entity: SiEntity,
  data: Record<string, any>,
  path: string[],
): void {
  const currentArray = _.get(data, path);
  for (let x = 0; x < currentArray.length; x++) {
    const indexValue = currentArray[x];
    const currentPath = _.concat(path, [`${x}`]);
    if (_.isArray(indexValue)) {
      entity.addOpSet({
        op: OpType.Set,
        source: OpSource.Inferred,
        path: currentPath,
        // @ts-ignore
        value: [],
        system: "baseline",
      });
      _k8sDiscoverEntityArray(entity, data, currentPath);
    } else if (_.isObjectLike(indexValue)) {
      entity.addOpSet({
        op: OpType.Set,
        source: OpSource.Inferred,
        path: currentPath,
        // @ts-ignore
        value: {},
        system: "baseline",
      });
      _k8sDiscoverEntityObject(entity, data, currentPath, indexValue);
    } else {
      entity.addOpSet({
        op: OpType.Set,
        source: OpSource.Inferred,
        path: currentPath,
        value: indexValue,
        system: "baseline",
      });
    }
  }
}

function _k8sDiscoverEntityObject(
  entity: SiEntity,
  data: Record<string, any>,
  path: string[],
  currentData?: Record<string, any>,
) {
  const walk = currentData ? currentData : data;
  for (const key of Object.keys(walk)) {
    const currentPath = _.concat(path, [key]);
    const currentValue = _.get(data, currentPath);
    debug({ currentPath, currentValue });
    if (_.isArray(currentValue)) {
      debug("is array");
      _k8sDiscoverEntityArray(entity, data, _.cloneDeep(currentPath));
    } else if (_.isObjectLike(currentValue)) {
      debug("is object");
      _k8sDiscoverEntityObject(
        entity,
        data,
        _.cloneDeep(currentPath),
        _.cloneDeep(currentValue),
      );
    } else {
      debug("is value");
      entity.addOpSet({
        op: OpType.Set,
        source: OpSource.Inferred,
        path: currentPath,
        value: currentValue,
        system: "baseline",
      });
    }
  }
}
