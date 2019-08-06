<template>
  <StandardLayout>
    <IntegrationFull :integrationInstanceId="this.$route.params.id" />
  </StandardLayout>
</template>

<script lang="ts">
import Vue from "vue";
import StandardLayout from "@/components/StandardLayout.vue";
import IntegrationFull from "@/components/IntegrationFull.vue";

export default Vue.extend({
  name: "IntegrationShow",
  data() {
    return {
      integrationInstanceId: "0",
    };
  },
  components: {
    StandardLayout,
    IntegrationFull,
  },
});
</script>
