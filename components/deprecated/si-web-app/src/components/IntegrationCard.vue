<template>
  <v-hover v-slot:default="{ hover }">
    <v-card
      :to="{ name: 'integration', params: { id: integrationInstanceId } }"
      :elevation="hover ? 12 : 2"
    >
      <v-img
        height="100"
        contain
        :src="require(`../assets/${integration.image}`)"
      ></v-img>
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

export default Vue.extend({
  name: "IntegrationCard",
  props: {
    integrationInstanceId: String,
    name: String,
    description: String,
    integration: Object,
  },
});
</script>
