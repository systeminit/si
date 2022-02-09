<template>
  <div v-if="schema">
    <div class="flex flex-col">
      <StatusBar />
      <SiChangeSetHeader>
        <template #title> Schema {{ schema.name }}</template>
      </SiChangeSetHeader>
    </div>
    <div>
      <EditForm
        :object-kind="EditFieldObjectKind.Schema"
        :object-id="schemaId"
      />
    </div>
  </div>
  <div v-else>
    <NotFound></NotFound>
  </div>
</template>

<script setup lang="ts">
import SiChangeSetHeader from "@/molecules/SiChangeSetHeader.vue";
import NotFound from "@/pages/NotFound.vue";
import { SchemaService } from "@/service/schema";
import { fromRef, refFrom } from "vuse-rx";
import { GlobalErrorService } from "@/service/global_error";
import { combineLatest, switchMap, from } from "rxjs";
import type { Schema } from "@/api/sdf/dal/schema";
import { EditFieldObjectKind } from "@/api/sdf/dal/edit_field";
import EditForm from "@/organisims/EditForm.vue";
import StatusBar from "@/molecules/StatusBar.vue";

const props = defineProps<{
  schemaId: number;
}>();

const props$ = fromRef(props, { immediate: true, deep: true });

const schema = refFrom<Schema | null>(
  combineLatest([props$]).pipe(
    switchMap(([props]) => {
      return SchemaService.getSchema({ schemaId: props.schemaId });
    }),
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([null]);
      } else {
        return from([response]);
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
