<template>
  <div>
    <template v-if="checkAuthReq.isPending">
      checking if you are logged in...
    </template>
    <template v-else>
      <nav>
        <template v-if="userIsLoggedIn">
          <p>hello {{ authStore.bestUserLabel }}!</p>
          <a class="button" :href="`${API_URL}/auth/logout`">Log out!</a>
        </template>
        <template v-else>
          <a class="button" :href="`${API_URL}/auth/login`">Log in!</a>
        </template>
      </nav>
      <RouterView />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeMount } from "vue";
import { RouterView } from "vue-router";
import { useAuthStore } from "./store/auth.store";

const authStore = useAuthStore();
const checkAuthReq = authStore.getRequestStatus("CHECK_AUTH");

const API_URL = import.meta.env.VITE_AUTH_API_URL;

const userIsLoggedIn = computed(() => authStore.userIsLoggedIn);

onBeforeMount(async () => {
  if (import.meta.env.SSR) return;
  await authStore.CHECK_AUTH();
});
</script>

<style scoped></style>
