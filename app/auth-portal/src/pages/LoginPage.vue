<template>
  <div>
    <h1>Login!</h1>
    <RouterLink :to="{ name: 'signup' }">Sign up</RouterLink>
  </div>
</template>

<script setup lang="ts">
import { onBeforeMount } from "vue";
import { RouterLink, useRouter } from "vue-router";
import { useAuthStore } from "@/store/auth.store";

const authStore = useAuthStore();
const router = useRouter();

onBeforeMount(async () => {
  if (authStore.userIsLoggedIn) {
    await router.push({ name: "dashboard" });
  }
});
</script>
