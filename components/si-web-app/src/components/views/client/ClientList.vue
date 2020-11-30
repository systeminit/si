<template>
  <div id="client-list" class="flex flex-col flex-no-wrap client-list-bg-color">
    <div class="flex flex-row h-10 mt-4 client-list-menu-bar">
      <button
        data-cy="new-client-button"
        class="h-8 px-2 mt-1 ml-4 text-white bg-teal-700 hover:bg-teal-600"
        @click="showModal()"
        type="button"
      >
        <div class="flex">
          <PlusSquareIcon size="1.25x" class="self-center text-gray-200" />
          <div class="ml-1 font-normal text-gray-100">new client</div>
        </div>
      </button>
    </div>

    <div class="mx-8 my-4">
      <table class="w-full table-fixed">
        <thead>
          <tr class="text-xs text-gray-200 client-table-title">
            <th class="w-1/2 px-4 py-2">Name</th>
            <th class="w-1/4 px-4 py-2">Type</th>
            <th class="w-1/4 px-4 py-2">Kind</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-800" v-if="clients">
          <tr v-for="client in clients" :key="client.name">
            <td class="client-table-row">
              <div class="px-4 py-2">{{ client.name }}</div>
            </td>
            <td class="client-table-row">
              <div class="px-4 py-2">{{ client.objectType }}</div>
            </td>
            <td class="client-table-row">
              <div class="px-4 py-2">{{ client.kind }}</div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <modal
      name="new-client"
      adaptive
      draggable
      styles="background-color:#313436"
    >
      <ClientNew />
    </modal>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import { registry } from "si-registry";
import { PlusSquareIcon } from "vue-feather-icons";
import SiSelect, { SelectProps } from "@/components/ui/SiSelect.vue";
import { ClientKind } from "@/api/sdf/model/client";
import ClientNew from "@/components/views/client/ClientNew.vue";

import { RootStore } from "../../../store";

export default Vue.extend({
  name: "ClientList",
  components: {
    PlusSquareIcon,
    ClientNew,
  },
  props: {
    organizationId: {
      type: String,
    },
    workspaceId: {
      type: String,
    },
  },
  methods: {
    showModal() {
      this.$modal.show("new-client");
    },
  },
  computed: {
    ...mapState({
      clients(state: RootStore): RootStore["client"]["clients"] {
        return state.client.clients;
      },
    }),
  },
});
</script>

<style scoped>
.client-list-menu-bar {
  background-color: #2d3031;
}
.client-list-bg-color {
  background-color: #212324;
}
.input-bg-color {
  background-color: #25788a;
}
.client-table-title {
  background-color: #292f32;
}
.client-table-row {
  @apply text-gray-300 text-xs text-center;
}
</style>
