<template>
  <div class="flex flex-col w-full page-background">
    <div
      class="flex flex-row items-center justify-between flex-grow-0 flex-shrink-0 h-12 header-background"
    >
      <div class="mt-1 ml-8 font-medium align-middle inline-flex items-center">
        <router-link
          :to="{ name: 'schema-list' }"
          class="inline-flex items-center"
        >
          <VueFeather type="chevron-left" />
        </router-link>
        Create Schema
      </div>
    </div>
    <div class="flex flex-row page-background">
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
              id="schemaKind"
              :options="kindOptions"
              value="concrete"
              size="xs"
            />
          </template>
        </SiFormRow>
        <div class="flex justify-end w-full">
          <div class="pr-2">
            <SiButton
              size="xs"
              label="Cancel"
              kind="cancel"
              icon="null"
              @click="cancel"
            />
          </div>
          <div>
            <SiButton
              size="xs"
              label="Create"
              kind="save"
              icon="null"
              @click="create"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import VueFeather from "vue-feather";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiSelect from "@/atoms/SiSelect.vue";
import SiButton from "@/atoms/SiButton.vue";
import SiFormRow from "@/atoms/SiFormRow.vue";
import SiError from "@/atoms/SiError.vue";

import { useRouter } from "vue-router";
import { ref } from "vue";
import { SchemaKind } from "@/api/sdf/dal/schema";
import { SchemaService } from "@/service/schema";
import { enumKeys } from "@/utils/enumKeys";

const router = useRouter();

const kindOptionsArray: Array<{ label: string; value: string }> = [];
for (const value of enumKeys(SchemaKind)) {
  kindOptionsArray.push({ label: value, value: SchemaKind[value] });
}
const kindOptions = ref(kindOptionsArray);

const form = ref({
  name: "",
  kind: SchemaKind.Concrete,
});

const cancel = async () => {
  await router.push({ name: "schema-list" });
};

const errorMessage = ref("");
const create = async () => {
  let result = await SchemaService.createSchema(form.value);
  if (result.error) {
    errorMessage.value = result.error.message;
  } else {
    await router.push({ name: "schema-list" });
  }
};

const _schemaNew = async () => {
  await router.push({ name: "schema-new" });
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
