<template>
  <section
    :class="
      clsx(
        'flex flex-col h-full',
        themeClasses(
          'bg-shade-0 text-shade-100',
          'bg-neutral-900 text-shade-0',
        ),
      )
    "
  >
    <header
      class="flex flex-row items-center px-sm py-xs border-b border-neutral-700"
    >
      <!-- Back button (smaller with no border) -->
      <button
        tabindex="-1"
        class="text-neutral-400 hover:text-white mr-xs flex flex-row items-center justify-center"
        aria-label="Back to actions list"
        @click="navigateBack"
      >
        <Icon name="arrow--left" size="sm" />
      </button>

      <!-- Function info -->
      <div class="flex-1 flex flex-row items-center gap-xs">
        <span class="font-medium">
          {{
            funcRun?.functionDisplayName ||
            funcRun?.functionName ||
            "Function Run"
          }}
        </span>

        <!-- add a bit more "gap" to the first (dt) element, so the pairs have more space between them -->
        <!-- negative bottom margin to pull it down so all items are visually aligned -->
        <dl
          class="text-sm text-neutral-400 flex flex-row items-center gap-2xs [&>dt]:ml-2xs mb-[-0.25em]"
        >
          <template v-if="funcRun?.functionKind">
            <dt><Icon name="func" size="xs" /></dt>
            <dd>{{ funcRun.functionKind }}</dd>
          </template>

          <template v-if="funcRun?.componentName">
            <dt><Icon name="component" size="xs" /></dt>
            <dd>{{ funcRun.componentName }}</dd>
          </template>

          <template v-if="funcRun?.actionKind">
            <dt><Icon name="play" size="xs" /></dt>
            <dd>{{ funcRun.actionKind }}</dd>
          </template>

          <!-- ID and Status -->
          <dt class="text-xs">ID:</dt>
          <dd class="text-xs">{{ funcRun.id }}</dd>
        </dl>
        <FuncRunStatusBadge v-if="funcRun && status" :status="status" />
      </div>

      <!-- Action buttons -->
      <div class="flex gap-2">
        <slot name="actions"></slot>
      </div>
    </header>

    <div
      class="grid grid-cols-[1fr_1fr] grid-rows-[1fr_1fr_1fr] gap-xs p-xs h-full overflow-hidden"
    >
      <GridItemWithLiveHeader
        class="row-span-3"
        title="Logs"
        :live="isLive && funcRun?.state === 'Running'"
      >
        <CodeViewer
          v-if="logText"
          ref="logsContainer"
          :code="logText"
          language="log"
          allowCopy
        />
        <div v-else class="text-neutral-400 italic text-xs p-xs">
          No logs available
        </div>
      </GridItemWithLiveHeader>

      <GridItemWithLiveHeader title="Code" :live="false">
        <CodeViewer
          v-if="functionCode"
          :code="functionCode"
          language="javascript"
          allowCopy
        />
        <div v-else class="text-neutral-400 italic text-xs p-xs">
          No code available
        </div>
      </GridItemWithLiveHeader>

      <GridItemWithLiveHeader title="Arguments" :live="false">
        <CodeViewer
          v-if="argsJson"
          :code="argsJson"
          language="json"
          allowCopy
        />
        <div v-else class="text-neutral-400 italic text-xs p-xs">
          No arguments available
        </div>
      </GridItemWithLiveHeader>

      <GridItemWithLiveHeader title="Result" :live="false">
        <CodeViewer
          v-if="resultJson"
          :code="resultJson"
          language="json"
          allowCopy
        />
        <div v-else class="text-neutral-400 italic text-xs p-xs">
          No result available
        </div>
      </GridItemWithLiveHeader>
    </div>
  </section>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import { useRoute, useRouter } from "vue-router";
import clsx from "clsx";
import CodeViewer from "@/components/CodeViewer.vue";
import { FuncRun } from "../api_composables/func_run";
import FuncRunStatusBadge from "../FuncRunStatusBadge.vue";
import GridItemWithLiveHeader from "./GridItemWithLiveHeader.vue";

const props = defineProps<{
  funcRun: FuncRun;
  status: string;
  logText: string;
  functionCode: string;
  argsJson: string;
  resultJson: string;
  isLive: boolean;
}>();

const router = useRouter();
const route = useRoute();
const logsContainer = ref<InstanceType<typeof CodeViewer> | null>(null);

// Navigate back to Explore view
const navigateBack = () => {
  router.push({
    name: "new-hotness",
    params: {
      workspacePk: route.params.workspacePk,
      changeSetId: route.params.changeSetId,
    },
  });
};

// Watch for log updates to scroll to bottom when they're loaded
watch(
  () => props.logText,
  (newText, oldText) => {
    if (newText && (!oldText || newText.length > oldText.length)) {
      // If logs were added, scroll to bottom after a short delay to allow rendering
      setTimeout(() => {
        if (logsContainer.value?.$el) {
          const codeElement =
            logsContainer.value.$el.querySelector(".overflow-auto");
          if (codeElement) {
            codeElement.scrollTop = codeElement.scrollHeight;
          }
        }
      }, 100);
    }
  },
);
</script>
