<template>
  <div>
    <h2 class="text-xl font-bold">Auth Dashboard!</h2>

    <ul>
      <li v-for="workspace in workspaces" :key="workspace.id">
        {{ workspace.displayName }}
        <VButton2
          icon="arrow--right"
          :href="`${API_HTTP_URL}/workspaces/${workspace.id}/go`"
        >
          Go to workspace
        </VButton2>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import { VButton2 } from "@si/vue-lib/design-system";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { API_HTTP_URL } from "@/store/api";

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();

const workspaces = computed(() => workspacesStore.workspaces);

function reloadWorkspaces() {
  if (import.meta.env.SSR) return;
  if (!authStore.userIsLoggedIn) return;

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  workspacesStore.LOAD_WORKSPACES();
}

watch(() => authStore.userIsLoggedIn, reloadWorkspaces, { immediate: true });
</script>
