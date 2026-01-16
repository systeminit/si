<template>
  <div
    v-if="
      selectedComponent && 'changeStatus' in selectedComponent.def && selectedComponent.def.changeStatus !== 'deleted'
    "
    class="h-full relative"
  >
    <ErrorMessage :requestStatus="diffReqStatus" />

    <template v-if="diffReqStatus.isSuccess && selectedComponentDiff">
      <div class="absolute inset-xs">
        <template v-if="selectedComponentDiff.diff?.code">
          <CodeViewer
            :code="selectedComponentDiff.diff?.code"
            :codeLanguage="selectedComponentDiff.diff?.language"
            :allowCopy="false"
          >
            <template #title>
              <span class="text-lg">Diff</span>
            </template>
          </CodeViewer>
        </template>
        <template v-else>
          <CodeViewer
            v-if="selectedComponentDiff.current.code"
            :code="selectedComponentDiff.current.code"
            :codeLanguage="selectedComponentDiff.current.language"
            :allowCopy="false"
          >
            <template #title>
              <span class="text-lg">Current</span>
            </template>
          </CodeViewer>
          <div v-else class="w-full text-center text-xl text-neutral-400 p-sm">No Code</div>
        </template>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import * as _ from "lodash-es";
import { ErrorMessage } from "@si/vue-lib/design-system";
import CodeViewer from "@/components/CodeViewer.vue";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useViewsStore } from "@/store/views.store";

const componentsStore = useComponentsStore();
const viewStore = useViewsStore();
const changeSetsStore = useChangeSetsStore();

const selectedComponentId = computed(() => viewStore.selectedComponentId);
const selectedComponent = computed(() => viewStore.selectedComponent);

const selectedComponentDiff = computed(() => componentsStore.componentDiffsById[viewStore.selectedComponentId || ""]);

const diffReqStatus = componentsStore.getRequestStatus("FETCH_COMPONENT_DIFF", selectedComponentId);

watch(
  [selectedComponentId, () => changeSetsStore.selectedChangeSetLastWrittenAt],
  () => {
    if (!selectedComponentId.value) return;
    componentsStore.FETCH_COMPONENT_DIFF(selectedComponentId.value);
  },
  { immediate: true },
);
</script>
