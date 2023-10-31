<template><div>Refresh auth</div></template>

<script setup lang="ts">
import { useRoute } from "vue-router";
import { onMounted } from "vue";

import { useAuthStore } from "@/store/auth.store";

const route = useRoute();

const authStore = useAuthStore();
const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;

const workspaceId = route.query.workspaceId as string;

onMounted(async () => {
  if (workspaceId) {
    await authStore.FORCE_REFRESH_MEMBERS(workspaceId);
  }
  window.location.href = `${AUTH_PORTAL_URL}/workspace/${workspaceId}`;
});
</script>
