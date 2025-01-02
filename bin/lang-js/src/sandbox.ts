import { Buffer } from "node:buffer";
import os from "node:os";
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";
import fetch from "node-fetch";
import toml from "toml";

import * as _ from "lodash-es";
import * as yaml from "js-yaml";

import Joi from "joi";
import { FunctionKind } from "./function.ts";
import { makeConsole } from "./sandbox/console.ts";
import { makeExec } from "./sandbox/exec.ts";
import * as assetBuilder from "./asset_builder.ts";
import {
  makeBeforeRequestStorage,
  makeMainRequestStorage,
} from "./sandbox/requestStorage.ts";
import { makePackage } from "./sandbox/package.ts";

export type Sandbox = Record<string, unknown>;

function commonSandbox(executionId: string): Sandbox {
  return {
    console: makeConsole(executionId),
    _,
    Buffer,
    requestStorage: makeMainRequestStorage(),
    zlib,
    fetch,
    siExec: makeExec(executionId),
    YAML: { stringify: yaml.dump, parse: yaml.load },
    os,
    fs,
    path,
    Joi,
    toml,
    pkg: makePackage(executionId),
  };
}

function schemaVariantDefinitionSandbox(): Sandbox {
  return {
    AssetBuilder: assetBuilder.AssetBuilder,
    PropBuilder: assetBuilder.PropBuilder,
    SecretDefinitionBuilder: assetBuilder.SecretDefinitionBuilder,
    SecretPropBuilder: assetBuilder.SecretPropBuilder,
    ValueFromBuilder: assetBuilder.ValueFromBuilder,
    SocketDefinitionBuilder: assetBuilder.SocketDefinitionBuilder,
    MapKeyFuncBuilder: assetBuilder.MapKeyFuncBuilder,
    PropWidgetDefinitionBuilder: assetBuilder.PropWidgetDefinitionBuilder,
    SiPropValueFromDefinitionBuilder:
      assetBuilder.SiPropValueFromDefinitionBuilder,
  };
}

function beforeFunctionSandbox(executionId: string): Sandbox {
  return {
    requestStorage: makeBeforeRequestStorage(executionId),
  };
}

export function createSandbox(
  kind: FunctionKind,
  executionId: string,
): Sandbox {
  let sandbox = commonSandbox(executionId);

  switch (kind) {
    case FunctionKind.SchemaVariantDefinition:
      sandbox = {
        ...sandbox,
        ...schemaVariantDefinitionSandbox(),
      };
      break;
    case FunctionKind.Before:
      sandbox = {
        ...sandbox,
        ...beforeFunctionSandbox(executionId),
      };
      break;
    default:
      break;
  }

  return sandbox;
}
