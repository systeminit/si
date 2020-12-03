<template>
  <div class="flex flex-col">
    <div
      class="flex items-center justify-between pl-1 text-sm text-white bg-black"
    >
      <div>Create new secret</div>
      <div>
        <button @click="hideModal" class="flex">
          <XIcon @click="hideModal"></XIcon>
        </button>
      </div>
    </div>

    <div class="p-4">
      <div class="flex flex-row mx-2 my-2">
        <div class="text-white">name:</div>

        <input
          data-cy="new-secret-form-secret-name"
          class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
          type="text"
          placeholder="secret name"
          v-model="secretName"
        />
      </div>
      <div class="flex flex-row mx-2 my-2">
        <div class="text-white">kind:</div>

        <SiSelect
          size="xs"
          class="mr-4"
          :options="secretKindList"
          v-model="secretKind"
          name="kind"
        />
      </div>

      <DockerHubCredential v-model="message" v-if="secretKind == 'dockerHub'" />
      <AwsAccessKeyCredential
        v-model="message"
        v-else-if="secretKind == 'awsAccessKey'"
      />
      <HelmRepoCredential
        v-model="message"
        v-else-if="secretKind == 'helmRepo'"
      />

      <div class="flex flex-row" v-if="secretKind">
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
</template>

<script lang="ts">
import Vue from "vue";
import { XIcon } from "vue-feather-icons";
import SiSelect, { SelectProps } from "@/components/ui/SiSelect.vue";
import DockerHubCredential from "@/components/views/secret/DockerHubCredential.vue";
import AwsAccessKeyCredential from "@/components/views/secret/AwsAccessKeyCredential.vue";
import HelmRepoCredential from "@/components/views/secret/HelmRepoCredential.vue";
import { SecretKind } from "@/api/sdf/model/secret";

interface Data {
  secretName: string;
  secretKind: string;
  secretKindList: SelectProps["options"];
  message: Record<string, any>;
}

export default Vue.extend({
  name: "SecretNew",
  components: {
    SiSelect,
    XIcon,
    DockerHubCredential,
    AwsAccessKeyCredential,
    HelmRepoCredential,
  },
  data(): Data {
    let secretKindList = [{ value: "", label: "none" }];
    for (const secretKind of Object.values(SecretKind)) {
      secretKindList.push({ value: secretKind, label: secretKind });
    }
    return {
      secretName: "",
      secretKind: "",
      secretKindList,
      message: {},
    };
  },
  methods: {
    async createSecret() {
      await this.$store.dispatch("secret/createCredential", {
        secretName: this.secretName,
        secretKind: this.secretKind,
        message: this.message,
      });
      this.hideModal();
      this.secretName = "";
      this.secretKind = "";
      this.message = {};
    },
    hideModal() {
      this.$modal.hide("new-secret");
    },
  },
  watch: {
    secretKind() {
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
