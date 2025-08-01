import { computed } from "vue";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";

// WARN(nick): this should only be used in "Workspace.vue".
export const useWorkspace = () => {
  // TODO(nick): yeet these! Keep usages of old stores sandboxed to composables to easily replace later.
  const authStore = useAuthStore();
  const workspacesStore = useWorkspacesStore();

  return {
    hasOneUser: computed(() => authStore.workspaceHasOneUser),
    approvalsEnabled: computed(() => workspacesStore.workspaceApprovalsEnabled),
    workspaceUsers: computed(() => authStore.workspaceUsers),
  };
};
