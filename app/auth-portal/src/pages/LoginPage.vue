<template>
  <RichText class="text-center">
    <h2>Welcome to System Initiative</h2>
    <p>
      Please click the button to log in or to signup if you don't have an SI
      account. You will be automatically redirected in 5 seconds.
    </p>
    <VButton2 :href="`${API_URL}/auth/login`" size="lg">
      Log in or Sign up!
    </VButton2>
  </RichText>
</template>

<script setup lang="ts">
import { onBeforeMount, onMounted } from "vue";
import { useRouter } from "vue-router";
import { RichText, VButton2 } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { useAuthStore } from "@/store/auth.store";

const API_URL = import.meta.env.VITE_AUTH_API_URL;

const authStore = useAuthStore();
const router = useRouter();

useHead({ title: "Login" });

onBeforeMount(async () => {
  if (authStore.userIsLoggedIn) {
    await router.push({ name: "login-success" });
  }
});

onMounted(() => {
  setTimeout(() => {
    window.location.replace(`${API_URL}/auth/login`);
  }, 5000);
});
</script>
