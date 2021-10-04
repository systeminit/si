import { From, Inference } from "si-inference";
import { OpSet, OpSource, OpType, SiEntity } from "si-entity";
import { findProp, Prop, registry } from "si-registry";
import Debug from "debug";
import _ from "lodash";
import { VM, VMScript } from "vm2";
import { FromEntry } from "si-inference";
import {
  InferenceError,
  InvalidFromPathForSchemaError,
  InvalidObjectKeysError,
  InvalidTargetPropError,
  InvalidToPathForSchemaError,
  UnexpectedInferenceToNameError,
  ValueTypeError,
} from "./errors";
import {
  ItemPropObject,
  PropObject,
} from "../../si-registry/dist/registryEntry";

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

export interface InferContextEntry {
  entity: SiEntity;
  secret?: Record<string, DecryptedSecret | null>;
}

export type InferContext = InferContextEntry[];

export interface EvaluateFromResult {
  inputs: SiEntity[];
  dataResult: DataResult;
  targetEntity?: SiEntity;
}

export function evaluateFrom(
  inference: Inference,
  targetEntity: SiEntity,
  context: InferContext,
): EvaluateFromResult {
  const result: { inputs: SiEntity[]; dataResult: DataResult } = {
    inputs: [],
    dataResult: { baseline: [] },
  };
  for (const selector of inference.from) {
    if (selector.entityType) {
      const selected = _.map(
        _.filter(context, (c) => c.entity.entityType == selector.entityType),
        (c) => c.entity,
      );
      populateData(inference, selector.data, selected, result.dataResult);
      result.inputs = _.union(result.inputs, selected);
    }
    if (selector.targetEntity) {
      const selected = [_.cloneDeep(targetEntity)];
      selected[0].isTarget = true;
      populateData(inference, selector.data, selected, result.dataResult);
      result.inputs = _.union(result.inputs, selected);
    }
    if (selector.entityId) {
      const selected = _.find(context, (c) => c.entity.id == selector.entityId);
      if (selected) {
        populateData(
          inference,
          selector.data,
          [selected.entity],
          result.dataResult,
        );
        result.inputs = _.union(result.inputs, [selected.entity]);
      }
    }
  }
  return result;
}

export type DataObject = {
  entityId: string;
  name?: string;
  properties: Record<string, any>;
};

export type DataResult = {
  [system: string]: DataObject[];
};

export function populateData(
  inference: Inference,
  dataFrom: From,
  inputs: SiEntity[],
  dataResult: DataResult,
): DataResult {
  //const dataResult: DataResult = { baseline: [] };
  let name: string | undefined = undefined;
  let fromList: Array<FromEntry> = [];

  if (_.isArray(dataFrom)) {
    fromList = dataFrom;
  } else {
    fromList = [dataFrom];
  }
  for (let x = 0; x < inputs.length; x++) {
    const entity = inputs[x];
    for (const fromEntry of fromList) {
      if (fromEntry.name) {
        name = _.cloneDeep(entity.name);
      } else {
        const fromProp = findProp([entity.entityType, ...fromEntry.path]);
        if (!fromProp) {
          throw new InvalidFromPathForSchemaError({
            inference,
            targetEntity: entity,
            path: fromEntry.path,
          });
        }
        const systemProperties = entity.getPropertyForAllSystems({
          path: fromEntry.path,
        });
        if (systemProperties) {
          for (const system of Object.keys(systemProperties)) {
            _.set(
              dataResult,
              [system, x, "properties", ...fromEntry.path],
              systemProperties[system],
            );
          }
        }
      }
    }
    for (const system of Object.keys(dataResult)) {
      if (!dataResult[system][x]) {
        dataResult[system][x] = {
          properties: {},
          entityId: entity.id,
        };
      } else {
        dataResult[system][x].entityId = entity.id;
      }
      if (name) {
        dataResult[system][x].name = name;
      }
    }
  }
  return dataResult;
}

function createVm(system: string, data: DataObject[]): VM {
  const debug = Debug("cyclone:inference:lambda");
  const forEachEntity = function forEachEntity(callback: (e: any) => void) {
    for (const e of data) {
      callback(e);
    }
  };
  const firstEntity = data[0] || {};
  const vm = new VM({
    timeout: 2000,
    sandbox: {
      debug,
      _,
      system,
      data,
      forEachEntity,
      firstEntity,
    },
    eval: false,
    wasm: false,
    fixAsync: true,
  });
  return vm;
}

function getTargetProp(
  inference: Inference,
  targetEntity: SiEntity,
): ReturnType<typeof findProp> | "name" {
  let targetProp: ReturnType<typeof findProp> | "name" = undefined;
  if (inference.to.path) {
    targetProp = findProp([targetEntity.entityType, ...inference.to.path]);
  } else if (inference.to.name) {
    targetProp = "name";
  }
  return targetProp;
}

function createProvenance({
  inputs,
  inference,
}: {
  inputs: SetValueOnTargetEntityArgs["inputs"];
  inference: Inference;
}): OpSet["provenance"] {
  const provenanceContext = _.map(inputs, (e) => {
    return { id: e.id, entityType: e.entityType };
  });
  const provenance = {
    context: provenanceContext,
    inference,
  };
  return provenance;
}

export interface CodeResult {
  code: VMScript;
  if?: VMScript;
}

function compileCode(inference: Inference): CodeResult {
  const code = new VMScript(inference.code);
  code.compile();
  const result: CodeResult = { code };

  if (inference.if) {
    const ifCode = new VMScript(inference.if);
    ifCode.compile();
    result.if = ifCode;
  }
  return result;
}

export function getPathFromInference(inference: Inference): string[] {
  if (inference.to.path) {
    if (inference.to.extraPath) {
      const newPath = _.cloneDeep(inference.to.path);
      newPath.push(...inference.to.extraPath);
      return newPath;
    }
    return inference.to.path;
  }
}

interface SetValueOnTargetEntityArgs {
  targetEntity: SiEntity;
  targetProp: ReturnType<typeof getTargetProp>;
  inference: Inference;
  inputs: ReturnType<typeof evaluateFrom>["inputs"];
  value: any;
  system: string;
  setOps: OpSet[];
}

function setValueOnTargetEntity(args: SetValueOnTargetEntityArgs): SiEntity {
  if (_.isUndefined(args.targetProp)) {
    throw new InvalidToPathForSchemaError({
      inference: args.inference,
      targetEntity: args.targetEntity,
    });
  } else if (args.targetProp == "name") {
    setNameOnTargetEntity(args);
  } else if (args.targetProp.type == "string") {
    setStringOnTargetEntity(args);
  } else if (args.targetProp.type == "number") {
    setNumberOnTargetEntity(args);
  } else if (args.targetProp.type == "boolean") {
    setBooleanOnTargetEntity(args);
  } else if (args.targetProp.type == "object") {
    setObjectOnTargetEntity(args);
  } else if (args.targetProp.type == "map") {
    setMapOnTargetEntity(args);
  } else if (args.targetProp.type == "array") {
    setArrayOnTargetEntity(args);
  }
  args.targetEntity.updateFromOps({
    inference: args.inference,
    setOps: args.setOps,
  });
  return args.targetEntity;
}

function setArrayOnTargetEntity({
  value,
  inputs,
  inference,
  targetEntity,
  targetProp,
  system,
  setOps,
}: SetValueOnTargetEntityArgs): SiEntity {
  if (targetProp != "name") {
    if (targetProp.type == "array") {
      if (_.isArray(value)) {
        const nextIndex = targetEntity.nextIndex(inference.to.path);
        for (let x = 0; x < value.length; x++) {
          const index = x + nextIndex;
          const newInference = _.cloneDeep(inference);
          if (newInference.to.path) {
            if (newInference.to.extraPath) {
              newInference.to.extraPath.push(`${index}`);
            } else {
              newInference.to.extraPath = [`${index}`];
            }
            const newTargetProp = targetProp.itemProperty;
            // @ts-ignore
            const newValue: any = value[index];
            if (newTargetProp && newValue) {
              setValueOnTargetEntity({
                value: newValue,
                inputs,
                inference: newInference,
                targetEntity,
                targetProp: newTargetProp,
                system,
                setOps,
              });
            }
          } else {
            throw new UnexpectedInferenceToNameError({
              targetEntity,
              targetType: "array",
              inference,
              value: value,
            });
          }
        }
      } else {
        throw new ValueTypeError({
          targetEntity,
          targetType: "array",
          inference,
          value: value,
        });
      }
    } else {
      throw new InvalidTargetPropError({
        expected: "array",
        found: targetProp.type,
      });
    }
  } else {
    throw new InvalidTargetPropError({ expected: "array", found: "name" });
  }
  return targetEntity;
}

function setMapOnTargetEntity({
  value,
  inputs,
  inference,
  targetEntity,
  targetProp,
  system,
  setOps,
}: SetValueOnTargetEntityArgs): SiEntity {
  if (targetProp != "name") {
    if (targetProp.type == "map") {
      if (_.isObject(value)) {
        const newKeys = Object.keys(value);
        for (const key of newKeys) {
          const newInference = _.cloneDeep(inference);
          if (newInference.to.path) {
            if (newInference.to.extraPath) {
              newInference.to.extraPath.push(key);
            } else {
              newInference.to.extraPath = [key];
            }
            const newTargetProp = targetProp.valueProperty;
            // @ts-ignore
            const newValue: any = value[key];
            if (newTargetProp && newValue) {
              setValueOnTargetEntity({
                value: newValue,
                inputs,
                inference: newInference,
                targetEntity,
                targetProp: newTargetProp,
                system,
                setOps,
              });
            }
          } else {
            throw new UnexpectedInferenceToNameError({
              targetEntity,
              targetType: "map",
              inference,
              value: value,
            });
          }
        }
      } else {
        throw new ValueTypeError({
          targetEntity,
          targetType: "map",
          inference,
          value: value,
        });
      }
    } else {
      throw new InvalidTargetPropError({
        expected: "map",
        found: targetProp.type,
      });
    }
  } else {
    throw new InvalidTargetPropError({ expected: "map", found: "name" });
  }
  return targetEntity;
}

function setObjectOnTargetEntity({
  value,
  inputs,
  inference,
  targetEntity,
  targetProp,
  system,
  setOps,
}: SetValueOnTargetEntityArgs): SiEntity {
  if (targetProp != "name") {
    if (targetProp.type == "object") {
      if (_.isObject(value)) {
        const newKeys = Object.keys(value);
        const validKeys = _.map(targetProp.properties, (p) => p.name);
        const invalidKeys = _.difference(newKeys, validKeys);
        if (invalidKeys.length != 0) {
          throw new InvalidObjectKeysError({
            targetEntity,
            targetType: "object",
            inference,
            value: value,
            invalidKeys,
            validKeys,
          });
        }
        for (const key of newKeys) {
          const newInference = _.cloneDeep(inference);
          if (newInference.to.path) {
            if (newInference.to.extraPath) {
              newInference.to.extraPath.push(key);
            } else {
              newInference.to.extraPath = [key];
            }
            // @ts-ignore
            const newValue: any = value[key];
            const newTargetProp = _.find(
              targetProp.properties,
              (p) => p.name == key,
            );
            if (newTargetProp && newValue) {
              setValueOnTargetEntity({
                value: newValue,
                inputs,
                inference: newInference,
                targetEntity,
                targetProp: newTargetProp,
                system,
                setOps,
              });
            }
          } else {
            throw new UnexpectedInferenceToNameError({
              targetEntity,
              targetType: "object",
              inference,
              value: value,
            });
          }
        }
      } else {
        throw new ValueTypeError({
          targetEntity,
          targetType: "object",
          inference,
          value: value,
        });
      }
    } else {
      throw new InvalidTargetPropError({
        expected: "object",
        found: targetProp.type,
      });
    }
  } else {
    throw new InvalidTargetPropError({ expected: "object", found: "name" });
  }
  return targetEntity;
}

function setStringOnTargetEntity({
  value,
  inputs,
  inference,
  targetEntity,
  system,
  setOps,
}: SetValueOnTargetEntityArgs): SiEntity {
  if (_.isString(value)) {
    const provenance = createProvenance({ inputs, inference });
    setOps.push({
      op: OpType.Set,
      source: OpSource.Inferred,
      path: getPathFromInference(inference),
      value,
      system,
      provenance,
    });
  } else {
    throw new ValueTypeError({
      targetEntity,
      targetType: "string",
      inference,
      value: value,
    });
  }
  return targetEntity;
}

function setNumberOnTargetEntity({
  value,
  inputs,
  inference,
  targetEntity,
  system,
  setOps,
}: SetValueOnTargetEntityArgs): SiEntity {
  const provenance = createProvenance({ inputs, inference });
  if (_.isString(value)) {
    if (_.isNaN(_.toNumber(value))) {
      throw new ValueTypeError({
        targetEntity,
        targetType: "number",
        inference,
        value: value,
      });
    }
    setOps.push({
      op: OpType.Set,
      source: OpSource.Inferred,
      path: getPathFromInference(inference),
      value,
      system,
      provenance,
    });
  } else if (_.isNumber(value)) {
    setOps.push({
      op: OpType.Set,
      source: OpSource.Inferred,
      path: getPathFromInference(inference),
      value: `${value}`,
      system,
      provenance,
    });
  } else {
    throw new ValueTypeError({
      targetEntity,
      targetType: "string",
      inference,
      value: value,
    });
  }
  return targetEntity;
}

function setBooleanOnTargetEntity({
  value,
  inputs,
  inference,
  targetEntity,
  system,
  setOps,
}: SetValueOnTargetEntityArgs): SiEntity {
  if (_.isBoolean(value)) {
    const provenance = createProvenance({ inputs, inference });
    setOps.push({
      op: OpType.Set,
      source: OpSource.Inferred,
      path: getPathFromInference(inference),
      value,
      system,
      provenance,
    });
  } else {
    throw new ValueTypeError({
      targetEntity,
      targetType: "boolean",
      inference,
      value,
    });
  }
  return targetEntity;
}

function setNameOnTargetEntity({
  targetEntity,
  inference,
  value,
}: SetValueOnTargetEntityArgs): SiEntity {
  if (_.isString(value)) {
    if (targetEntity.name.startsWith("si-")) {
      targetEntity.name = value;
    }
  } else {
    throw new ValueTypeError({
      targetEntity,
      targetType: "string",
      inference,
      value,
    });
  }
  return targetEntity;
}

export function evaluateInferenceLambda(
  inference: Inference,
  targetEntity: SiEntity,
  context: InferContext,
): SiEntity {
  const debug = Debug("cyclone:inference:lambda");
  const { inputs, dataResult } = evaluateFrom(inference, targetEntity, context);
  const targetProp = getTargetProp(inference, targetEntity);
  const compiled = compileCode(inference);

  for (const system of Object.keys(dataResult)) {
    const data = dataResult[system];
    const vm = createVm(system, data);
    if (compiled.if) {
      const ifResult = vm.run(compiled.if);
      if (!ifResult) {
        debug(
          `lambda has an if condition that returned false for system ${system}. Existing inference will be removed, and no new values will be set.`,
        );
        targetEntity.updateFromOps({ inference, setOps: [] });
        return;
      }
    }
    const value = vm.run(compiled.code);
    if (_.isUndefined(value)) {
      debug(
        `lambda returned undefined for system ${system}. Existing inference will be removed`,
        {
          inputs,
          data,
        },
      );
      targetEntity.updateFromOps({ inference, setOps: [] });
    } else {
      setValueOnTargetEntity({
        targetEntity,
        targetProp,
        inference,
        inputs,
        value,
        system,
        setOps: [],
      });
    }
  }
  return targetEntity;
}
