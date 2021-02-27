import { findProp, Validator, ValidatorKind } from "si-registry";
import validatorjs from "validator";

interface ValidateSuccess {
  errors?: never;
  success: true;
}

interface ValidateFailure {
  success?: never;
  errors: string[];
}

export type ValidateResult = ValidateSuccess | ValidateFailure;

export function validate(path: string[], value: string): ValidateResult {
  const prop = findProp(path);
  if (!prop) {
    return {
      errors: [`Bug! Cannot find property to validate! Path was: ${path}`],
    };
  }
  let errors: string[] = [];
  if (prop.validation) {
    const result = runValidators(prop.validation, value);
    if (result.errors) {
      errors = errors.concat(result.errors);
    }
  }
  if (prop.type == "array") {
    if (prop.itemProperty.validation) {
      const result = runValidators(prop.itemProperty.validation, value);
      if (result.errors) {
        errors = errors.concat(result.errors);
      }
    }
  }
  if (errors.length > 0) {
    return { errors };
  }
  return { success: true };
}

export function runValidators(
  validators: Validator[],
  value: string,
): ValidateResult {
  const errors = [];
  for (const validator of validators) {
    if (validator.kind == ValidatorKind.Alphanumeric) {
      if (!validatorjs.isAlphanumeric(value)) {
        errors.push("string must be alphanumeric (a-zA-Z0-9)");
      }
    }
    if (errors.length > 0) {
      return { errors };
    }
  }
  return { success: true };
}
