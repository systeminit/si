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
      <template v-slot:item.supportedActions="{ item }">
        <template>
          {{ item.supportedActions.join(", ") }}
        </template>
      </template>
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
        <td
          :colspan="props.headers.length"
          v-else-if="props.item.nodeType == 'Operating System'"
        >
          <ul>
            <li>Operating System Name: {{ props.item.operatingSystemName }}</li>
            <li>
              Operating System Version: {{ props.item.operatingSystemVersion }}
            </li>
            <li>
              Operating System Release: {{ props.item.operatingSystemRelease }}
            </li>
            <li>Platform: {{ props.item.platform }}</li>
            <li>Platform Version: {{ props.item.platformVersion }}</li>
            <li>Platform Release: {{ props.item.platformRelease }}</li>
            <li>
              Supported Architectures: {{ props.item.architecture.join(", ") }}
            </li>
          </ul>
        </td>
        <td :colspan="props.headers.length" v-else>
          This is a bug; add the node type to the slot for expansion
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
      getComponents: [],
      headers: [
        { text: "Node Type", value: "nodeType" },
        { text: "Name", align: "left", value: "name" },
        { text: "Description", value: "description" },
        { text: "Supported Actions", value: "supportedActions" },
      ],
    };
  },
  props: {
    integrationId: String,
    workspaceId: String,
  },
});
</script>
