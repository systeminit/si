<template>
  <div class="flex flex-col h-screen bg-black justify-center items-center">
    <meta name="robots" content="noindex" />
    <div class="text-center">
      <p class="text-white text-5xl">Page Not Found</p>
      <p class="text-white text-xl">We tried, though. Seriously.</p>
      <p class="text-xl text-white">
        Sorry about that, but there is nothing here.
      </p>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "NotFoundPage",
});
</script>
