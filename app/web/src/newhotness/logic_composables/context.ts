import { inject } from "vue";
import { assertIsDefined, Context } from "../types";

export function useContext(): Context {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);
  return ctx;
}
