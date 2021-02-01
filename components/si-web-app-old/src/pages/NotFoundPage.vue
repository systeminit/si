<template>
  <CenteredLayout>
    <meta name="robots" content="noindex" />
    <div>
      <p class="text-white text-5xl">Page Not Found</p>
      <p class="text-white text-xl">We tried, though. Seriously.</p>
      <p class="text-white text-xl">
        Sorry about that, but there is nothing here.
      </p>
    </div>
  </CenteredLayout>
</template>

<script lang="ts">
import Vue from "vue";
import CenteredLayout from "@/components/layout/CenteredLayout.vue";

export default Vue.extend({
  name: "signin",
  components: {
    CenteredLayout,
  },
});
</script>
