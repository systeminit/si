<template>
  <div class="flex flex-col">
    <div
      class="flex items-center justify-between pl-1 text-sm text-white bg-black"
    >
      <div>
        Create new client
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
          data-cy="new-client-form-client-name"
          class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
          type="text"
          placeholder="client name"
          v-model="clientName"
        />
      </div>
      <div class="flex flex-row mx-2 my-2">
        <div class="text-white">
          kind:
        </div>

        <SiSelect
          size="xs"
          class="mr-4"
          :options="clientKindList"
          v-model="clientKind"
          name="kind"
        />
      </div>

      <ApiClient v-model="message" v-if="clientKind == 'apiClient'" />

      <div class="flex flex-row" v-if="clientKind">
        <button
          data-cy="new-client-form-create-button"
          class="w-16 mt-4 ml-4 text-white bg-teal-700 hover:bg-teal-600"
          @click="createClient"
          type="button"
        >
          create
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { XIcon } from "vue-feather-icons";
import SiSelect, { SelectProps } from "@/components/ui/SiSelect.vue";
import ApiClient from "@/components/views/client/ApiClient.vue";
import { ClientKind } from "@/api/sdf/model/client";

interface Data {
  clientName: string;
  clientKind: string;
  clientKindList: SelectProps["options"];
  message: Record<string, any>;
}

export default Vue.extend({
  name: "ClientNew",
  components: {
    SiSelect,
    XIcon,
    ApiClient,
  },
  data(): Data {
    let clientKindList = [{ value: "", label: "none" }];
    for (const clientKind of Object.values(ClientKind)) {
      console.log(clientKind);
      clientKindList.push({ value: clientKind, label: clientKind });
    }
    return {
      clientName: "",
      clientKind: "",
      clientKindList,
      message: {},
    };
  },
  methods: {
    async createClient() {
      await this.$store.dispatch("client/createClient", {
        clientName: this.clientName,
        clientKind: this.clientKind,
        message: this.message,
      });
      this.hideModal();
      this.clientName = "";
      this.clientKind = "";
      this.message = {};
    },
    hideModal() {
      this.$modal.hide("new-client");
    },
  },
  watch: {
    clientKind() {
      this.message = {};
    },
  },
});
</script>

<style scoped>
.input-bg-color {
  background-color: #25788a;
}
</style>
