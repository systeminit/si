import os from "os";
import fs from "fs";
import path from "path";
import zlib from "zlib";
import fetch from "node-fetch";

import * as _ from "lodash-es";
import * as yaml from "js-yaml";

import Joi from "joi";
import { FunctionKind } from "./function";
import { makeConsole } from "./sandbox/console";
import { makeExec } from "./sandbox/exec";
import * as assetBuilder from "./asset_builder";
import {
  makeBeforeRequestStorage,
  makeMainRequestStorage,
} from "./sandbox/requestStorage";

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
    SiPropValueFromDefinitionBuilder: assetBuilder.SiPropValueFromDefinitionBuilder,
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
