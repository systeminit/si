<template>
  <v-hover v-slot:default="{ hover }">
    <v-card
      :to="{ name: 'workspace', params: { id: id } }"
      :elevation="hover ? 12 : 2"
    >
      <v-card-title>
        {{ name }}
      </v-card-title>
      <v-divider />
      <v-card-text>{{ description }}</v-card-text>
    </v-card>
  </v-hover>
</template>

<script lang="ts">
import Vue from "vue";

import getWorkspaces from "@/graphql/queries/getWorkspaces.graphql";

export default Vue.extend({
  name: "WorkspaceCard",
  props: {
    id: String,
    name: String,
    description: String,
  },
});
</script>
