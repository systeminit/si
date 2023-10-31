<template>
  <div
    v-if="selectedComponent && selectedComponent.changeStatus !== 'deleted'"
    class="h-full relative"
  >
    <ErrorMessage :requestStatus="diffReqStatus" />

    <template v-if="diffReqStatus.isSuccess && selectedComponentDiff">
      <div class="absolute inset-xs">
        <template v-if="selectedComponent.changeStatus === 'unmodified'">
          <CodeViewer
            :code="selectedComponentDiff.current.code || 'No code'"
            :codeLanguage="selectedComponentDiff.current.language"
          >
            <template #title>
              <span class="text-lg">Current</span>
            </template>
          </CodeViewer>
        </template>
        <template v-else>
          <!-- what to do about multiple diffs? -->
          <CodeViewer
            :code="selectedComponentDiff.diffs[0]?.code || 'No code'"
            :codeLanguage="selectedComponentDiff.diffs[0]?.language"
          >
            <template #title>
              <span class="text-lg">Diff</span>
            </template>
          </CodeViewer>
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

const componentsStore = useComponentsStore();
const changeSetsStore = useChangeSetsStore();

const selectedComponentId = computed(() => componentsStore.selectedComponentId);
const selectedComponent = computed(() => componentsStore.selectedComponent);

const selectedComponentDiff = computed(
  () => componentsStore.selectedComponentDiff,
);

const diffReqStatus = componentsStore.getRequestStatus(
  "FETCH_COMPONENT_DIFF",
  selectedComponentId,
);

watch(
  [selectedComponentId, () => changeSetsStore.selectedChangeSetWritten],
  () => {
    if (!selectedComponentId.value) return;
    componentsStore.FETCH_COMPONENT_DIFF(selectedComponentId.value);
  },
  { immediate: true },
);
</script>
