<template>
  <div>
    <h2>Auth Dashboard!</h2>

    <ul>
      <li v-for="workspace in workspaces" :key="workspace.id">
        {{ workspace.displayName }}
        <a
          class="button"
          :href="`${API_HTTP_URL}/workspaces/${workspace.id}/go`"
          >Go</a
        >
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router";
import { computed, watch } from "vue";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { API_HTTP_URL } from "@/store/api";

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();

const workspaces = computed(() => workspacesStore.workspaces);

const router = useRouter();

function reloadWorkspaces() {
  if (!authStore.userIsLoggedIn) return;

  // might want to show in a modal, but we'll see...
  if (authStore.user?.needsTosUpdate) {
    return router.push({ name: "review-tos" });
  }

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  workspacesStore.LOAD_WORKSPACES();
}

watch(() => authStore.userIsLoggedIn, reloadWorkspaces, { immediate: true });
</script>
