<template>
  <div class="m-auto text-center p-2xl">
    <template v-if="checkAuthReq.isPending">
      checking if you are logged in...
    </template>
    <template v-else>
      <nav>
        <template v-if="userIsLoggedIn">
          <p>hello {{ authStore.bestUserLabel }}!</p>
          <VButton2
            tone="neutral"
            icon="x"
            variant="ghost"
            :href="`${API_URL}/auth/logout`"
            >Log out!</VButton2
          >
        </template>
        <template v-else>
          <VButton2 :href="`${API_URL}/auth/login`">Log in!</VButton2>
        </template>
      </nav>
      <RouterView />
    </template>
  </div>
</template>

<script setup lang="ts">
import { VButton2 } from "@si/vue-lib/design-system";
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

<style></style>
