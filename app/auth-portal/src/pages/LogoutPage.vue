<template>
  <div>
    <h1>Logging you out...</h1>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";

import { useAuthStore } from "@/store/auth.store";

const authStore = useAuthStore();

onMounted(async () => {
  if (authStore.userIsLoggedIn) {
    await authStore.logout();
  }
});
</script>
