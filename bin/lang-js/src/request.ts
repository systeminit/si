import { ActionRunFunc } from "./function_kinds/action_run";
import { BeforeFunc } from "./function_kinds/before";
import { JoiValidationFunc } from "./function_kinds/joi_validation";
import { ReconciliationFunc } from "./function_kinds/reconciliation";
import { ResolverFunc } from "./function_kinds/resolver_function";
import {
  SchemaVariantDefinitionFunc,
} from "./function_kinds/schema_variant_definition";

export type AnyFunction =
  | ActionRunFunc
  | JoiValidationFunc
  | BeforeFunc
  | ReconciliationFunc
  | ResolverFunc
  | SchemaVariantDefinitionFunc;

export type Request = AnyFunction &
RequestCtx & {
  before?: BeforeFunc[];
};

export interface RequestCtx {
  executionId: string;
}

export const ctxFromRequest = ({ executionId }: Request): RequestCtx => ({
  executionId,
});
