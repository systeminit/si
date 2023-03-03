<template>
  <div>
    <h1>404 not found :(</h1>
    <RouterLink :to="{ name: 'login' }">Back to login page</RouterLink>
  </div>
</template>

<script setup lang="ts">
import { RouterLink } from "vue-router";
</script>
