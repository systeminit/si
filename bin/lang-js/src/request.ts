import { ActionRunFunc } from "./function_kinds/action_run";
import { BeforeFunc } from "./function_kinds/before";
import { JoiValidationFunc } from "./function_kinds/joi_validation";
import { ResolverFunc } from "./function_kinds/resolver_function";
import { ManagementFunc } from "./function_kinds/management";
import {
  SchemaVariantDefinitionFunc,
} from "./function_kinds/schema_variant_definition";

export type AnyFunction =
  | ActionRunFunc
  | JoiValidationFunc
  | BeforeFunc
  | ResolverFunc
  | SchemaVariantDefinitionFunc
  | ManagementFunc;

export type Request = AnyFunction &
RequestCtx & {
  before?: BeforeFunc[];
  timeout?: number,
};

export interface RequestCtx {
  executionId: string;
}

export const ctxFromRequest = ({ executionId }: Request): RequestCtx => ({
  executionId,
});
