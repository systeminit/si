<template>
  <div v-if="isLoaded()">
    <div class="flex flex-col">
      <StatusBar />
      <SiChangeSetHeader>
        <template #title> Schema {{ schema?.name || "" }}</template>
      </SiChangeSetHeader>
    </div>
    <div>
      <EditForm
        :object-kind="EditFieldObjectKind.Schema"
        :object-id="schemaId"
      />
    </div>
  </div>
  <div v-else-if="isNotFound()">
    <NotFound></NotFound>
  </div>
</template>

<script setup lang="ts">
import SiChangeSetHeader from "@/molecules/SiChangeSetHeader.vue";
import NotFound from "@/pages/NotFound.vue";
import { SchemaService } from "@/service/schema";
import { refFrom } from "vuse-rx";
import { GlobalErrorService } from "@/service/global_error";
import type { Schema } from "@/api/sdf/dal/schema";
import * as Rx from "rxjs";
import { ref } from "vue";
import EditForm from "@/organisims/EditForm.vue";
import StatusBar from "@/molecules/StatusBar.vue";
import { EditFieldObjectKind } from "@/api/sdf/dal/edit_field";
import { switchMap } from "rxjs/operators";

const props = defineProps({
  schemaId: {
    type: Number,
    required: true,
  },
});

enum ReadyState {
  LOADING,
  LOADED,
  NOT_FOUND,
}

const ready = ref<ReadyState>(ReadyState.LOADING);
const isLoaded = () => ready.value == ReadyState.LOADED;
const isNotFound = () => ready.value == ReadyState.NOT_FOUND;

const schema = refFrom<Schema | null>(
  SchemaService.getSchema({ schemaId: props.schemaId.valueOf() }).pipe(
    switchMap((schema) => {
      if (schema) {
        if (schema.error) {
          GlobalErrorService.set(schema);
          ready.value = ReadyState.NOT_FOUND;
          return Rx.from([null]);
        } else {
          ready.value = ReadyState.LOADED;
          return Rx.from([schema]);
        }
      } else {
        return Rx.from([null]);
      }
    }),
  ),
);
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
