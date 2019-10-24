<template>
  <v-list-item>
    <v-list-item-action>
      <v-switch @click.stop="toggleIntegration()" v-model="enabled"></v-switch>
    </v-list-item-action>
    <v-list-item-title v-if="integrationName">
      {{ integrationName }}: {{ title }}
    </v-list-item-title>
    <v-list-item-title v-else>{{ title }}</v-list-item-title>
  </v-list-item>
</template>

<script lang="ts">
import Vue from "vue";

import enableIntegrationInstanceOnWorkspace from "@/graphql/mutation/enableIntegrationInstanceOnWorkspace.graphql";
import disableIntegrationInstanceOnWorkspace from "@/graphql/mutation/disableIntegrationInstanceOnWorkspace.graphql";

export default Vue.extend({
  name: "IntegratonToggle",
  methods: {
    toggleIntegration() {
      if (this.enabled === true) {
        this.$apollo.mutate({
          mutation: disableIntegrationInstanceOnWorkspace,
          variables: {
            integrationInstanceId: this.integrationInstanceId,
            workspaceId: this.workspaceId,
          },
          update: (store, toggleData) => {
            this.enabled = false;
          },
        });
      } else {
        this.$apollo.mutate({
          mutation: enableIntegrationInstanceOnWorkspace,
          variables: {
            integrationInstanceId: this.integrationInstanceId,
            workspaceId: this.workspaceId,
          },
          update: (store, toggleData) => {
            this.enabled = true;
          },
        });
      }
    },
  },
  data() {
    return {
      enabled: this.initialEnabled,
    };
  },
  props: {
    integrationInstanceId: String,
    workspaceId: String,
    initialEnabled: Boolean,
    title: String,
    integrationName: String,
  },
});
</script>
