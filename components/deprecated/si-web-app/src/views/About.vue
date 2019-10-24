<template>
  <StandardLayout>
    <div class="about">
      <h1>This is an about page</h1>
    </div>
  </StandardLayout>
</template>

<script lang="ts">
import Vue from "vue";
import StandardLayout from "@/components/StandardLayout.vue";

export default Vue.extend({
  name: "about",
  components: {
    StandardLayout,
  },
});
</script>
