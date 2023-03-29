<template>
  <RichText class="text-center">
    <h2>Welcome to System Initiative</h2>
    <p>Please click the button to log in</p>
    <VButton2 :href="`${API_URL}/auth/login`" size="lg">Log in!</VButton2>
  </RichText>
</template>

<script setup lang="ts">
import { onBeforeMount } from "vue";
import { useRouter } from "vue-router";
import { RichText, VButton2 } from "@si/vue-lib/design-system";
import { useAuthStore } from "@/store/auth.store";

const API_URL = import.meta.env.VITE_AUTH_API_URL;

const authStore = useAuthStore();
const router = useRouter();

onBeforeMount(async () => {
  if (authStore.userIsLoggedIn) {
    await router.push({ name: "login-success" });
  }
});
</script>
