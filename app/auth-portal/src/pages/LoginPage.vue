<template>
  <RichText class="text-center">
    <h2>Welcome to System Initiative</h2>
    <p>
      You are being redirected to Auth0 to complete login/signup. If you are not
      automatically redirected, please
      <a :href="AUTHORIZE_URL">click here</a> to continue.
    </p>

    <!-- <p class="italic">
      <template v-if="countDownSeconds === 0">Redirecting...</template>
      <template v-else>
        You will be automatically redirected in {{ countDownSeconds }}
        {{ countDownSeconds === 1 ? "second" : "seconds" }}
      </template>
    </p> -->
  </RichText>
</template>

<script setup lang="ts">
import { onBeforeMount, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { RichText } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { useAuthStore } from "@/store/auth.store";

const authStore = useAuthStore();
const router = useRouter();
const route = useRoute();

const IS_SIGNUP = route.name === "signup";

const API_URL = import.meta.env.VITE_AUTH_API_URL;
// if on /signup we add an extra hint to tell auth0 to start on signup
const AUTHORIZE_URL = `${API_URL}/auth/login${IS_SIGNUP ? "?signup=1" : ""}`;

// const countDownSeconds = ref(0);

useHead({ title: IS_SIGNUP ? "Sign Up" : "Login" });

onBeforeMount(async () => {
  if (authStore.userIsLoggedIn) {
    await router.push({ name: "login-success" });
  }
});

onMounted(() => {
  window.location.replace(AUTHORIZE_URL);
  // setInterval(() => {
  //   // in case redirecting fails or takes longer, dont want the timer to go negative
  //   if (countDownSeconds.value === 0) return;
  //   countDownSeconds.value--;
  //   if (countDownSeconds.value === 0) {
  //     window.location.replace(AUTHORIZE_URL);
  //   }
  // }, 1000);
});
</script>
