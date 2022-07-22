<template>
  <div class="flex flex-col">
    <SiError :message="errorMessage" />
    <div class="flex flex-col mt-4">
      <SiFormRow>
        <template #label>
          <label>Schema Name:</label>
        </template>
        <template #widget>
          <SiTextBox
            id="schemaName"
            v-model="form.name"
            data-test="schema-new-form-name"
            size="xs"
            name="name"
            placeholder="schema name"
            :is-show-type="false"
            required
          />
        </template>
      </SiFormRow>
      <SiFormRow>
        <template #label>
          <label>Schema Kind:</label>
        </template>
        <template #widget>
          <SiSelect
            id="schema-new-form-kind"
            data-test="schema-new-form-kind"
            :options="kindOptions"
            :model-value="form.kind"
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
            data-test="schema-new-form-create-button"
            size="xs"
            label="Create"
            kind="save"
            :icon="null"
            @click="create"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiSelect from "@/atoms/SiSelect.vue";
import SiButton from "@/atoms/SiButton.vue";
import SiFormRow from "@/atoms/SiFormRow.vue";
import SiError from "@/atoms/SiError.vue";

import { ref } from "vue";
import { SchemaKind } from "@/api/sdf/dal/schema";
import { SchemaService } from "@/service/schema";
import { enumKeys } from "@/utils/enumKeys";

const emit = defineEmits(["cancel", "create"]);

const kindOptionsArray: Array<{ label: string; value: string }> = [];
for (const value of enumKeys(SchemaKind)) {
  kindOptionsArray.push({ label: value, value: SchemaKind[value] });
}
const kindOptions = ref(kindOptionsArray);

const form = ref({
  name: "",
  kind: SchemaKind.Concrete,
});

const errorMessage = ref("");

const create = () => {
  SchemaService.createSchema(form.value).subscribe((response) => {
    if (response.error) {
      errorMessage.value = response.error.message;
    }
    emit("create", response);
  });
};

const cancel = () => {
  form.value.name = "";
  form.value.kind = SchemaKind.Concrete;
  emit("cancel");
};
</script>

<style scoped>
.page-background {
  background-color: #1e1e1e;
}

.header-background {
  background-color: #171717;
}
</style>
