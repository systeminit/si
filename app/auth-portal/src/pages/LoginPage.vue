<template>
  <div>
    <p>Click the button to log in...</p>
  </div>
</template>

<script setup lang="ts">
import { onBeforeMount } from "vue";
import { useRouter } from "vue-router";
import { useAuthStore } from "@/store/auth.store";

const authStore = useAuthStore();
const router = useRouter();

onBeforeMount(async () => {
  if (authStore.userIsLoggedIn) {
    await router.push({ name: "dashboard" });
  }
});
</script>
