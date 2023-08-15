<template>
  <div>
    <h1>You have successfully logged out</h1>
    <p>
      Redirecting you back to
      <a href="https://www.systeminit.com">systeminit.com</a>.
    </p>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";

onMounted(async () => {
  window.location.replace("https://www.systeminit.com");
});
</script>
