import { Buffer } from "node:buffer";
import os from "node:os";
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";

import toml from "npm:toml";
import jsonpatch from "npm:fast-json-patch";
import * as _ from "https://deno.land/x/lodash_es@v0.0.2/mod.ts";
import * as yaml from "npm:js-yaml";
import Joi from "npm:joi";

import { FunctionKind } from "./function.ts";
import { makeExec } from "./sandbox/exec.ts";
import layout from "./sandbox/layout.ts";
import template from "./sandbox/template.ts";
import * as assetBuilder from "./asset_builder.ts";
import {
  makeBeforeRequestStorage,
  makeMainRequestStorage,
} from "./sandbox/requestStorage.ts";

export type Sandbox = Record<string, unknown>;

function commonSandbox(executionId: string): Sandbox {
  return {
    _,
    Buffer,
    requestStorage: makeMainRequestStorage(),
    zlib,
    siExec: makeExec(executionId),
    YAML: { stringify: yaml.dump, parse: yaml.load },
    os,
    fs,
    path,
    Joi,
    toml,
    jsonpatch,
    layout,
    template,
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
