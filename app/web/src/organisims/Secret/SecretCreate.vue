<template>
  <div class="flex flex-col w-full">
    <SiError
      v-if="placeholderShowError"
      :test="placeholderString"
      :message="placeholderString"
      :success="true"
      @clear="placeholderFunc"
    />
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="placeholderString">Secret Name:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          id="placeholderString"
          v-model="placeholderForm.secretName"
          size="xs"
          name="secretName"
          placeholder="secret name"
          :is-show-type="false"
          required
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="placeholderString">Secret Kind:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiSelect
          id="placeholderString"
          v-model="selectedSecretKind"
          size="xs"
          name="secretKind"
          :options="secretKinds"
          required
        />
      </div>
    </div>

    <DockerHubCredential
      v-if="selectedSecretKind === SecretKind.DockerHubCredential"
      @input="placeholderFunc"
    />

    <div class="flex justify-end w-full">
      <div class="pr-2">
        <SiButton
          size="xs"
          label="Cancel"
          kind="cancel"
          icon="null"
          @click="placeholderFunc"
        />
      </div>
      <div>
        <SiButton
          size="xs"
          label="Create"
          kind="save"
          icon="null"
          @click="placeholderFunc"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SiButton from "@/atoms/SiButton.vue";
import SiError from "@/atoms/SiError.vue";
import SiSelect, { SelectPropsOption } from "@/atoms/SiSelect.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import DockerHubCredential from "@/organisims/Secret/Create/DockerHubCredential.vue";
import { ref } from "vue";

enum SecretKind {
  DockerHubCredential = "DockerHub Credential",
  Empty = "",
}

// FIXME(nick): change this to only be calculated once and dynamically loaded.
const secretKinds: SelectPropsOption[] = [
  {
    value: SecretKind.DockerHubCredential,
    label: SecretKind.DockerHubCredential,
  },
  { value: SecretKind.Empty, label: SecretKind.Empty },
];

const selectedSecretKind = ref<SecretKind>(SecretKind.Empty);

const placeholderShowError = false;
const placeholderString = "ilikemybutt";
const placeholderSecretKind = SecretKind.DockerHubCredential;
const placeholderFunc = () => {
  console.log(placeholderString);
};
const placeholderForm = {
  secretName: placeholderString,
  secretKind: SecretKind.DockerHubCredential,
  message: placeholderString,
};
</script>

<style scoped>
.background {
  background-color: #1e1e1e;
}

.header {
  background-color: #3a3d40;
}

.row-item {
  background-color: #262626;
}

.row-item:nth-child(odd) {
  background-color: #2c2c2c;
}

.table-border {
  border-bottom: 1px solid #46494d;
}
</style>
