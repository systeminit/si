<template>
  <div>
    <h1>Logging you out...</h1>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter } from "vue-router";
import { useAuthStore } from "@/store/auth.store";

const authStore = useAuthStore();
const router = useRouter();

onMounted(async () => {
  if (authStore.userIsLoggedIn) {
    await authStore.logout();
  } else {
    await router.push({ name: "login" });
  }
});
</script>
