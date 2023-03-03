<template>
  <div>
    <h1>Signup!</h1>
    <RouterLink :to="{ name: 'login' }">Log in</RouterLink>
  </div>
</template>

<script setup lang="ts">
import { RouterLink } from "vue-router";
</script>
