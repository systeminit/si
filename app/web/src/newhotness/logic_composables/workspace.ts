import { computed, inject } from "vue";
import { assertIsDefined, Context } from "../types";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";

export const useWorkspace = () => {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);

  // TODO(nick): yeet these! Keep usages of old stores sandboxed to composables to easily replace
  // later.
const authStore = useAuthStore();
const workspacesStore =useWorkspacesStore();

  return {
    hasOneUser: computed(() => authStore.workspaceHasOneUser),
    approvalsEnabled: computed(() => workspacesStore.workspaceApprovalsEnabled),
    workspaceUsers: computed(() => authStore.workspaceUsers),
  }
};
