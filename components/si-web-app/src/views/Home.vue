<template>
  <StandardLayout>
    <v-container class="fill-height" fluid>
      <v-row justify="center" align="center">
        <v-col class="text-center">
          <WorkspaceTop />
        </v-col>
      </v-row>
    </v-container>
  </StandardLayout>
</template>

<script lang="ts">
import Vue from "vue";
import StandardLayout from "@/components/StandardLayout.vue";
import WorkspaceTop from "@/components/WorkspaceTop.vue";

export default Vue.extend({
  name: "home",
  components: {
    StandardLayout,
    WorkspaceTop,
  },
});
</script>
