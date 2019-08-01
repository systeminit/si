<template>
  <StandardLayout>
    <IntegrationTop />
  </StandardLayout>
</template>

<script lang="ts">
import Vue from "vue";
import StandardLayout from "@/components/StandardLayout.vue";
import IntegrationTop from "@/components/IntegrationTop.vue";

export default Vue.extend({
  name: "Integrations",
  components: {
    StandardLayout,
    IntegrationTop,
  },
});
</script>

