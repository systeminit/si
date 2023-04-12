<template>
  <RichText class="text-center">
    <h2>Welcome to System Initiative</h2>
    <p>
      Please click the button to log in or to signup if you don't have an SI
      account.
    </p>

    <p class="italic">
      <template v-if="countDownSeconds === 0">Redirecting...</template>
      <template v-else>
        You will be automatically redirected in {{ countDownSeconds }}
        {{ countDownSeconds === 1 ? "second" : "seconds" }}
      </template>
    </p>
    <VButton2 :href="LOGIN_URL" size="lg">Log in or Sign up!</VButton2>
  </RichText>
</template>

<script setup lang="ts">
import { onBeforeMount, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { RichText, VButton2 } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { useAuthStore } from "@/store/auth.store";

const API_URL = import.meta.env.VITE_AUTH_API_URL;
const LOGIN_URL = `${API_URL}/auth/login`;

const authStore = useAuthStore();
const router = useRouter();

const countDownSeconds = ref(5);

useHead({ title: "Login" });

onBeforeMount(async () => {
  if (authStore.userIsLoggedIn) {
    await router.push({ name: "login-success" });
  }
});

onMounted(() => {
  setInterval(() => {
    // in case redirecting fails or takes longer, dont want the timer to go negative
    if (countDownSeconds.value === 0) return;

    countDownSeconds.value--;
    if (countDownSeconds.value === 0) {
      window.location.replace(LOGIN_URL);
    }
  }, 1000);
});
</script>
