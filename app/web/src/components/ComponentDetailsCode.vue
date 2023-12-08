<template>
  <div>
    <template v-if="codeReqStatus.isPending"> Loading code...</template>
    <template v-else-if="codeReqStatus.isError">
      <ErrorMessage :requestStatus="codeReqStatus" />
    </template>
    <template v-else-if="codeReqStatus.isSuccess && selectedComponentCode">
      <div v-if="selectedComponentCode[0]?.code" class="absolute inset-xs">
        <template v-for="(item, index) in selectedComponentCode" :key="index">
          <div v-if="item.code || item.message" class="pb-md">
            <div
              v-if="selectedComponentCode.length > 1"
              class="text-lg font-bold pb-xs px-xs"
            >
              Code Output {{ index + 1 }}:
            </div>
            <ErrorMessage v-if="item.message" class="mx-1 mb-2">
              {{ item.message }}
            </ErrorMessage>
            <div class="relative">
              <CodeViewer v-if="item.code" :code="item.code" />
            </div>
          </div>
        </template>
      </div>
      <div
        v-else
        class="flex flex-row items-center justify-center p-md text-lg italic text-neutral-500 dark:text-neutral-400"
      >
        No code generated yet
      </div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { ErrorMessage } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import CodeViewer from "./CodeViewer.vue";

const componentsStore = useComponentsStore();

const selectedComponentId = computed(
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  () => componentsStore.selectedComponentId!,
);

const codeReqStatus = componentsStore.getRequestStatus(
  "FETCH_COMPONENT_CODE",
  selectedComponentId,
);

const selectedComponentCode = computed(
  () => componentsStore.selectedComponentCode,
);
</script>
