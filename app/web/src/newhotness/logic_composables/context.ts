import { inject } from "vue";
import { assertIsDefined, Context } from "../types";

// This will only work in Vue components which are descendants of the newhotness Workspace!
// DO NOT USE THIS IN WORKSPACE OR INSIDE OF OTHER LOGIC COMPOSABLES
export function useContext(): Context {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);
  return ctx;
}
