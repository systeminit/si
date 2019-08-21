<template>
  <v-card>
    <v-card-title>
      Components
      <div class="flex-grow-1"></div>
      <v-text-field
        v-model="search"
        append-icon="search"
        label="Search"
        single-line
        hide-details
      ></v-text-field>
    </v-card-title>
    <v-data-table
      :headers="headers"
      :items="getComponents"
      :search="search"
      :expanded.sync="expanded"
      show-expand
    >
      <template v-slot:expanded-item="props">
        <td
          :colspan="props.headers.length"
          v-if="props.item.nodeType == 'Server'"
        >
          <ul>
            <li>CPU: {{ props.item.cpu.name }}</li>
            <li>Cores: {{ props.item.cpu.cores }}</li>
            <li>Base Frequency: {{ props.item.cpu.baseFreqMHz }} MHz</li>
            <li>
              All Core Turbo Frequency:
              {{ props.item.cpu.allCoreTurboFreqMHz }} MHz
            </li>
            <li>
              Single Core Turbo Frequency:
              {{ props.item.cpu.singleCoreTurboFreqMHz }} MHz
            </li>
            <li>Memory {{ props.item.memoryGIB }} GIB</li>
          </ul>
        </td>
      </template>
    </v-data-table>
  </v-card>
</template>

<script lang="ts">
import Vue from "vue";

import getComponents from "@/graphql/queries/getComponents.graphql";

export default Vue.extend({
  name: "ComponentTable",
  apollo: {
    getComponents: {
      query: getComponents,
      variables() {
        return {
          integrationId: this.integrationId,
          workspaceId: this.workspaceId,
        };
      },
    },
  },
  data() {
    return {
      expanded: [],
      search: "",
      headers: [
        { text: "Name", align: "left", value: "name" },
        { text: "Description", value: "description" },
        { text: "Node Type", value: "nodeType" },
      ],
    };
  },
  props: {
    integrationId: String,
    workspaceId: String,
  },
});
</script>
