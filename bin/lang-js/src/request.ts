import { ActionRunFunc } from "./function_kinds/action_run.ts";
import { BeforeFunc } from "./function_kinds/before.ts";
import { JoiValidationFunc } from "./function_kinds/joi_validation.ts";
import { ResolverFunc } from "./function_kinds/resolver_function.ts";
import { ManagementFunc } from "./function_kinds/management.ts";
import {
  SchemaVariantDefinitionFunc,
} from "./function_kinds/schema_variant_definition.ts";

export type AnyFunction =
  | ActionRunFunc
  | JoiValidationFunc
  | BeforeFunc
  | ResolverFunc
  | SchemaVariantDefinitionFunc
  | ManagementFunc;

export type Request =
  & AnyFunction
  & RequestCtx
  & {
    before?: BeforeFunc[];
    timeout?: number;
  };

export interface RequestCtx {
  executionId: string;
}

export const ctxFromRequest = ({ executionId }: Request): RequestCtx => ({
  executionId,
});
