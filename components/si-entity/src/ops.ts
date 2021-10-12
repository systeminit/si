import { Inference } from "si-inference";
import { findProp, registry } from "si-registry";
import { v4 as uuidv4 } from "uuid";
import { InvalidOpPathError } from "./errors";
import _ from "lodash";

export enum OpSource {
  Manual = "manual",
  Inferred = "inferred",
}

export interface IOpBase {
  id: string;
  type: string;
  source: OpSource;
  path: string[];
  system: "baseline" | string;
}

export interface OpSetBase extends IOpBase {
  type: "set";
  value:
    | string
    | number
    | boolean
    | Record<string, unknown>
    | Array<unknown>
    | null;
  editPartial?: string;
}

export interface OpSetManual extends OpSetBase {
  source: OpSource.Manual;
  provenance?: never;
}

export interface OpSetInferred extends OpSetBase {
  source: OpSource.Inferred;
  provenance: {
    context: {
      id: string;
      entityType: string;
    }[];
    inference: Inference;
  };
}

export type OpSet = OpSetManual | OpSetInferred;

export function validatePathForEntityType(args: {
  path: OpSet["path"];
  entityType: string;
}): boolean {
  const fullPath = [args.entityType, ...args.path];
  const prop = findProp(fullPath);
  if (prop) {
    return true;
  } else {
    return false;
  }
}

export interface OpSetCreateManualArgs {
  entityType: string;
  system: OpSet["system"];
  path: OpSet["path"];
  value: OpSet["value"];
  editPartial?: string;
}

function createManual({
  entityType,
  value,
  path,
  system,
  editPartial,
}: OpSetCreateManualArgs): OpSetManual {
  const id = uuidv4();
  if (!validatePathForEntityType({ path, entityType })) {
    throw new InvalidOpPathError({ entityType, path });
  }
  return {
    id,
    type: "set",
    source: OpSource.Manual,
    value,
    system,
    path,
    editPartial,
  };
}

export interface OpSetCreateInferredArgs {
  entityType: string;
  system: OpSet["system"];
  path: OpSet["path"];
  value: OpSet["value"];
  editPartial?: OpSet["editPartial"];
  provenance: OpSet["provenance"];
}

function createInferred({
  entityType,
  value,
  path,
  system,
  editPartial,
  provenance,
}: OpSetCreateInferredArgs): OpSetInferred {
  const id = uuidv4();
  if (!validatePathForEntityType({ path, entityType })) {
    throw new InvalidOpPathError({ entityType, path });
  }
  return {
    id,
    type: "set",
    source: OpSource.Inferred,
    value,
    system,
    path,
    editPartial,
    provenance,
  };
}

export interface IsEqualOpSet {
  path: OpSet["path"];
  value: OpSet["value"];
  system: OpSet["system"];
  source: OpSet["source"];
}

function isEqual(left: IsEqualOpSet, right: IsEqualOpSet): boolean {
  const pathIsEqual = _.isEqual(left.path, right.path);
  const valueIsEqual = _.isEqual(left.value, right.value);
  const systemIsEqual = _.isEqual(left.system, right.system);
  const sourceIsEqual = _.isEqual(left.source, right.source);

  return pathIsEqual && valueIsEqual && systemIsEqual && sourceIsEqual;
}

function isEqualExceptValue(left: IsEqualOpSet, right: IsEqualOpSet): boolean {
  const pathIsEqual = _.isEqual(left.path, right.path);
  const systemIsEqual = _.isEqual(left.system, right.system);
  const sourceIsEqual = _.isEqual(left.source, right.source);

  return pathIsEqual && systemIsEqual && sourceIsEqual;
}

export const OpSet = {
  createManual,
  createInferred,
  isEqual,
  isEqualExceptValue,
};
