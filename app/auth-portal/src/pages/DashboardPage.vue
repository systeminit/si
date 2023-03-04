<template>
  <div>
    <h1>Dashboard!</h1>

    <ul>
      <li v-for="workspace in workspaces" :key="workspace.id">
        {{ workspace.displayName }}
        <a :href="`${API_HTTP_URL}/workspaces/${workspace.id}/go`">Go</a>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { RouterLink } from "vue-router";
import { computed, onBeforeMount } from "vue";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { API_HTTP_URL } from "@/store/api";

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();

const workspaces = computed(() => workspacesStore.workspaces);

onBeforeMount(async () => {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  workspacesStore.LOAD_WORKSPACES();
});
</script>
