import { SiEntity } from "si-entity";
import { Inference } from "si-inference";
import { getPathFromInference } from ".";

export class InferenceError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "InferenceError";
  }
}

export class RegistryLookupError extends InferenceError {
  constructor(args: { entityType: string }) {
    const message = `Entity type ${args.entityType} does not exist in the registry!`;
    super(message);
    this.name = "RegistryLookupError";
  }
}

export class InvalidTargetPropError extends InferenceError {
  constructor(args: { expected: string; found: string }) {
    const message = `Invalid target prop type in value function; expected '${args.expected}' found '${args.found}'`;
    super(message);
    this.name = "InvalidTargetPropError";
  }
}

export interface ValueTypeErrorConstructor {
  targetEntity: SiEntity;
  targetType: string;
  inference: Inference;
  value: any;
}

export class ValueTypeError extends InferenceError {
  constructor(args: ValueTypeErrorConstructor) {
    let message: string;
    const path = getPathFromInference(args.inference);
    if (args.inference.to.path) {
      message = `Inference '${args.inference.name}' for ${
        args.targetEntity.entityType
      }[${path.join(", ")}] expects a ${
        args.targetType
      }; received ${JSON.stringify(args.value)}`;
    } else {
      message = `Inference '${args.inference.name}' for ${
        args.targetEntity.entityType
      }.name expects a ${args.targetType}; received ${JSON.stringify(
        args.value,
      )}`;
    }
    super(message);
    this.name = "ValueTypeError";
  }
}

export class UnexpectedInferenceToNameError extends InferenceError {
  constructor(args: ValueTypeErrorConstructor) {
    const message = `Inference '${args.inference.name}' for ${
      args.targetEntity.entityType
    }[${args.inference.to.path.join(
      ", ",
    )}] expected a 'to' path, but instead found a name`;
    super(message);
    this.name = "UnexpectedInferenceToNameError";
  }
}

export interface InvalidObjectKeysErrorConstructor
  extends ValueTypeErrorConstructor {
  invalidKeys: string[];
  validKeys: string[];
}

export class InvalidObjectKeysError extends InferenceError {
  constructor(args: InvalidObjectKeysErrorConstructor) {
    const message = `Inference '${args.inference.name}' for object ${
      args.targetEntity.entityType
    }[${args.inference.to.path?.join(
      ", ",
    )}] has invalid keys: ${args.invalidKeys.join(
      ", ",
    )} (valid keys: ${args.validKeys.join(", ")})`;
    super(message);
    this.name = "ValueTypeError";
  }
}

export class InvalidToPathForSchemaError extends InferenceError {
  constructor(args: { inference: Inference; targetEntity: SiEntity }) {
    const path = getPathFromInference(args.inference);
    const message = `Inference '${args.inference.name}' for object ${
      args.targetEntity.entityType
    } has an invalid 'to' path: [${path.join(
      ", ",
    )}]; inference and schema must match!`;
    super(message);
    this.name = "InvalidToPathForSchemaError";
  }
}

export class InvalidFromPathForSchemaError extends InferenceError {
  constructor(args: {
    inference: Inference;
    targetEntity: SiEntity;
    path: string[];
  }) {
    const message = `Inference '${args.inference.name}' for object ${
      args.targetEntity.entityType
    } has an invalid 'from' path: [${args.path.join(
      ", ",
    )}]; inference and schema must match!`;
    super(message);
    this.name = "InvalidFromPathForSchemaError";
  }
}
