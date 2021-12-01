<template>
  <div class="flex flex-col w-full page-background">
    <SiChangeSetHeader>
      <template #title>
        <router-link
          :to="{ name: 'schema-list' }"
          class="inline-flex items-center"
        >
          <VueFeather type="chevron-left" />
        </router-link>
        Create Schema
      </template>
    </SiChangeSetHeader>
    <div class="flex flex-row">
      <SchemaCreateForm @create="create" @cancel="cancel" />
    </div>
  </div>
</template>

<script setup lang="ts">
import VueFeather from "vue-feather";
import SchemaCreateForm from "./SchemaCreateForm.vue";

import { useRouter } from "vue-router";
import { GlobalErrorService } from "@/service/global_error";
import { ApiResponse } from "@/api/sdf";
import { CreateSchemaResponse } from "@/service/schema/create_schema";
import SiChangeSetHeader from "@/molecules/SiChangeSetHeader.vue";

const router = useRouter();

const cancel = async () => {
  await router.push({ name: "schema-list" });
};

const create = async (result: ApiResponse<CreateSchemaResponse>) => {
  if (result.error) {
    GlobalErrorService.set(result);
  } else {
    await router.push({ name: "schema-list" });
  }
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
