<template>
  <div>
    <h1>Login!</h1>
    <RouterLink :to="{ name: 'signup' }">Sign up</RouterLink>
  </div>
</template>

<script setup lang="ts">
import { RouterLink } from "vue-router";
</script>
