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
    window.location.href = `${import.meta.env.VITE_AUTH_API_URL}/auth/logout`;
    await router.push({ name: "dashboard" });
  }
});
</script>
