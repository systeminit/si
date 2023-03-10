<template>
  <div>
    <template v-if="checkAuthReq.isPending">
      checking if you are logged in...
    </template>
    <template v-else>
      <nav>
        <template v-if="userIsLoggedIn">
          <p>hello {{ authStore.bestUserLabel }}!</p>
          <a href="http://localhost:9001/auth/logout">Log out!</a>
        </template>
        <template v-else>
          <a href="http://localhost:9001/auth/login">Log in!</a>
        </template>
      </nav>
      <RouterView />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeMount, onMounted } from "vue";
import { RouterView } from "vue-router";
import { useAuthStore } from "./store/auth.store";

const authStore = useAuthStore();
const checkAuthReq = authStore.getRequestStatus("CHECK_AUTH");

const userIsLoggedIn = computed(() => authStore.userIsLoggedIn);

onBeforeMount(async () => {
  await authStore.CHECK_AUTH();
});
</script>

<style scoped></style>
