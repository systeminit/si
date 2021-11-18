<template>
  <div class="flex flex-col">
    <div
      class="flex flex-row items-center justify-between flex-grow-0 flex-shrink-0 h-12 header-background"
    >
      <div class="mt-1 ml-8 font-medium align-middle">
        Schema
        <SiButton
          class="ml-2"
          icon="plus"
          label="New"
          size="xs"
          :disabled="!editMode"
          @click="schemaNew()"
        />
      </div>
      <SiModal
        v-if="modal"
        v-model="schemaCreateModalShow"
        name="schemaCreate"
        :esc-to-close="true"
      >
        <template #title>Create Schema</template>
        <template #body>
          <SchemaCreateForm
            @create="closeCreateSchemaModal"
            @cancel="closeCreateSchemaModal"
          />
        </template>
        <template #buttons>
          <div></div>
        </template>
      </SiModal>
      <div class="mt-1 mr-8 align-middle">
        <ChangeSetWidget />
      </div>
    </div>
    <div class="flex flex-row mt-5 page-background w-full">
      <div class="flex flex-col w-full">
        <div class="flex flex-row">
          <div class="w-6/12 px-2 py-1 text-center align-middle table-border">
            Name
          </div>
          <div class="w-6/12 px-2 py-1 text-center align-middle table-border">
            Kind
          </div>
        </div>
        <div
          v-for="schema in schemaList"
          :key="schema.pk"
          class="flex flex-row row-item"
        >
          <div class="w-6/12 px-2 py-1 text-center">
            {{ schema.name }}
          </div>
          <div class="w-6/12 px-2 py-1 text-center align-middle">
            {{ schema.kind }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SiButton from "@/atoms/SiButton.vue";
import SiModal from "@/molecules/SiModal.vue";
import ChangeSetWidget from "@/organisims/ChangeSetWidget.vue";
import SchemaCreateForm from "./SchemaCreateForm.vue";

import { Schema } from "@/api/sdf/dal/schema";
import { GlobalErrorService } from "@/service/global_error";

import { useRouter } from "vue-router";
import { ref } from "vue";
import { refFrom, untilUnmounted } from "vuse-rx";
import { from } from "rxjs";
import { switchMap } from "rxjs/operators";
import { SchemaService } from "@/service/schema";
import { ChangeSetService } from "@/service/change_set";

const router = useRouter();

const schemaCreateModalShow = ref(false);

const props = defineProps({
  modal: {
    type: Boolean,
    default: true,
  },
});

const editMode = refFrom(ChangeSetService.currentEditMode());
const schemaList = refFrom<Array<Schema>>(
  SchemaService.listSchemas().pipe(
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([[]]);
      } else {
        return from([response.list]);
      }
    }),
  ),
);

const schemaNew = async () => {
  if (props.modal.valueOf()) {
    schemaCreateModalShow.value = true;
  } else {
    await router.push({ name: "schema-new" });
  }
};

const closeCreateSchemaModal = () => {
  schemaCreateModalShow.value = false;
};
</script>

<style scoped>
.page-background {
  background-color: #1e1e1e;
}

.header-background {
  background-color: #171717;
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
