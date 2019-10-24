<template>
  <CenteredLayout>
    <meta name="robots" content="noindex" />
    <v-card class="elevation-12">
      <v-card-title>Page Not Found</v-card-title>
      <v-card-subtitle>We tried, though. Seriously.</v-card-subtitle>
      <v-card-text>Sorry about that, but there is nothing here.</v-card-text>
    </v-card>
  </CenteredLayout>
</template>

<script lang="ts">
import Vue from "vue";

import CenteredLayout from "@/components/CenteredLayout.vue";

export default Vue.extend({
  name: "signin",
  components: {
    CenteredLayout,
  },
});
</script>
