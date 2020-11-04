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
      width="500"
      height="250"
      draggable
      styles="background-color:#313436"
    >
      <div class="flex flex-col">
        <div
          class="flex items-center justify-between pl-1 text-sm text-white bg-black"
        >
          <div>
            Create new secret
          </div>
          <div>
            <button @click="hideModal" class="flex">
              <XIcon @click="hideModal"></XIcon>
            </button>
          </div>
        </div>

        <div class="p-4">
          <div class="flex flex-row mx-2 my-2">
            <div class="text-white">
              name:
            </div>

            <input
              data-cy="new-secret-form-secret-name"
              class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
              type="text"
              placeholder="secret name"
              v-model="secretName"
            />
          </div>
          <div class="flex flex-row mx-2 my-2">
            <div class="text-white">
              kind:
            </div>

            <SiSelect
              size="xs"
              class="mr-4"
              :options="secretKindList"
              v-model="secretKind"
              name="kind"
            />
          </div>
          <div class="flex flex-row mx-2 my-2">
            <div class="text-white">
              dockerHub username:
            </div>

            <input
              data-cy="new-secret-form-secret-name"
              class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
              type="text"
              placeholder="docker hub username"
              v-model="dockerHubUsername"
            />
          </div>
          <div class="flex flex-row mx-2 my-2">
            <div class="text-white">
              dockerHub password:
            </div>

            <input
              data-cy="new-secret-form-secret-name"
              class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
              type="password"
              placeholder="docker hub password"
              v-model="dockerHubPassword"
            />
          </div>
          <div class="flex flex-row">
            <button
              data-cy="new-secret-form-create-button"
              class="w-16 mt-4 ml-4 text-white bg-teal-700 hover:bg-teal-600"
              @click="createSecret"
              type="button"
            >
              create
            </button>
          </div>
        </div>
      </div>
    </modal>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import { registry } from "si-registry";
import { PlusSquareIcon, XIcon } from "vue-feather-icons";
import SiSelect, { SelectProps } from "@/components/ui/SiSelect.vue";
import { SecretKind } from "@/api/sdf/model/secret";

import { RootStore } from "../../../store";

interface Data {
  secretName: string;
  secretKind: string;
  secretKindList: SelectProps["options"];
  dockerHubUsername: string;
  dockerHubPassword: string;
}

export default Vue.extend({
  name: "SecretList",
  components: {
    PlusSquareIcon,
    SiSelect,
    XIcon,
  },
  props: {
    organizationId: {
      type: String,
    },
    workspaceId: {
      type: String,
    },
  },
  data(): Data {
    return {
      secretName: "",
      secretKind: "",
      secretKindList: [
        { value: SecretKind.DockerHub, label: SecretKind.DockerHub },
      ],
      dockerHubUsername: "",
      dockerHubPassword: "",
    };
  },
  methods: {
    async createSecret() {
      await this.$store.dispatch("secret/createDockerHubCredential", {
        secretName: this.secretName,
        dockerHubUsername: this.dockerHubUsername,
        dockerHubPassword: this.dockerHubPassword,
      });
      this.hideModal();
      this.secretName = "";
      this.secretKind = "";
      this.dockerHubUsername = "";
      this.dockerHubPassword = "";
    },
    showModal() {
      this.$modal.show("new-secret");
    },
    hideModal() {
      this.$modal.hide("new-secret");
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
