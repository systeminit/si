<template>
  <div>
    <h2 class="text-xl font-bold">Auth Dashboard!</h2>

    <button @click="ddm?.open">GO</button>
    <DropdownMenu ref="ddm">
      <DropdownMenuItem icon="trash">Item 1</DropdownMenuItem>
      <DropdownMenuItem>Item 2</DropdownMenuItem>
    </DropdownMenu>

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
import { useRouter } from "vue-router";
import { computed, ref, watch } from "vue";
import {
  DropdownMenu,
  DropdownMenuItem,
  Icon,
  VButton2,
} from "@si/vue-lib/design-system";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { API_HTTP_URL } from "@/store/api";

const ddm = ref<InstanceType<typeof DropdownMenu>>();

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();

const workspaces = computed(() => workspacesStore.workspaces);

const router = useRouter();

function reloadWorkspaces() {
  if (import.meta.env.SSR) return;
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
