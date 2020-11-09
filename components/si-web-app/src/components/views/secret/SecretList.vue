<template>
  <div id="secret-list" class="flex flex-col flex-no-wrap secret-list-bg-color">
    <div class="flex flex-row h-10 mt-4 secret-list-menu-bar">
      <button
        data-cy="new-secret-button"
        class="h-8 px-2 mt-1 ml-4 text-white bg-teal-700 hover:bg-teal-600"
        @click="showModal()"
        type="button"
      >
        <div class="flex">
          <PlusSquareIcon size="1.25x" class="self-center text-gray-200" />
          <div class="ml-1 font-normal text-gray-100">new secret</div>
        </div>
      </button>
    </div>

    <div class="mx-8 my-4">
      <table class="w-full table-fixed">
        <thead>
          <tr class="text-xs text-gray-200 secret-table-title">
            <th class="w-1/2 px-4 py-2">Name</th>
            <th class="w-1/4 px-4 py-2">Type</th>
            <th class="w-1/4 px-4 py-2">Kind</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-800" v-if="secrets">
          <tr v-for="secret in secrets" :key="secret.name">
            <td class="secret-table-row">
              <div class="px-4 py-2">{{ secret.name }}</div>
            </td>
            <td class="secret-table-row">
              <div class="px-4 py-2">{{ secret.objectType }}</div>
            </td>
            <td class="secret-table-row">
              <div class="px-4 py-2">{{ secret.kind }}</div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <modal
      name="new-secret"
      adaptive
      draggable
      styles="background-color:#313436"
    >
      <SecretNew />
    </modal>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import { registry } from "si-registry";
import { PlusSquareIcon } from "vue-feather-icons";
import SiSelect, { SelectProps } from "@/components/ui/SiSelect.vue";
import { SecretKind } from "@/api/sdf/model/secret";
import SecretNew from "@/components/views/secret/SecretNew.vue";

import { RootStore } from "../../../store";

export default Vue.extend({
  name: "SecretList",
  components: {
    PlusSquareIcon,
    SecretNew,
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
      this.$modal.show("new-secret");
    },
  },
  computed: {
    ...mapState({
      secrets(state: RootStore): RootStore["secret"]["secrets"] {
        return state.secret.secrets;
      },
    }),
  },
});
</script>

<style scoped>
.secret-list-menu-bar {
  background-color: #2d3031;
}
.secret-list-bg-color {
  background-color: #212324;
}
.input-bg-color {
  background-color: #25788a;
}
.secret-table-title {
  background-color: #292f32;
}
.secret-table-row {
  @apply text-gray-300 text-xs text-center;
}
</style>
