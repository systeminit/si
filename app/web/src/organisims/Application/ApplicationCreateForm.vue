<template>
  <div class="flex flex-col w-full">
    <SiError :message="errorMessage" />
    <SiFormRow>
      <template #label>
        <label>Application Name</label>
      </template>
      <template #widget>
        <SiTextBox
          id="applicationName"
          v-model="form.applicationName"
          name="applicationName"
          placeholder="super dope"
          required
          size="sm"
          @keyup.enter="onEnter"
          @keyup.escape="cancel"
        />
      </template>
    </SiFormRow>
    <div class="flex justify-end w-full">
      <div class="pr-2">
        <SiButton
          size="xs"
          label="Cancel"
          kind="cancel"
          :icon="null"
          @click="cancel"
        />
      </div>
      <div>
        <SiButton
          size="xs"
          label="Create"
          kind="save"
          :icon="null"
          @click="create"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

import SiButton from "@/atoms/SiButton.vue";
import SiError from "@/atoms/SiError.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiFormRow from "@/atoms/SiFormRow.vue";
import { ApplicationService } from "@/service/application";

const emit = defineEmits(["cancel", "create"]);

const errorMessage = ref<string>("");
const form = ref({
  applicationName: "",
});

const create = () => {
  ApplicationService.createApplication({
    name: form.value.applicationName,
  }).subscribe((response) => {
    if (response.error) {
      errorMessage.value = response.error.message;
    }
    form.value.applicationName = "";
    emit("create", response);
  });
};

const onEnter = () => {
  if (form.value.applicationName.length > 0) {
    create();
  }
};
const cancel = () => {
  form.value.applicationName = "";
  emit("cancel");
};
</script>
