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
      :class="
        clsx(
          'flex flex-row items-center px-sm py-xs border-t border-b border-neutral-600',
          themeClasses('bg-neutral-200', 'bg-neutral-800'),
        )
      "
    >
      <IconButton
        tooltip="Close (Esc)"
        tooltipPlacement="top"
        class="border-0 mr-2em"
        icon="x"
        size="sm"
        iconIdleTone="shade"
        iconTone="shade"
        @click="navigateBack"
      />

      <!-- Function info -->
      <div class="flex-1 flex flex-row items-center gap-xs">
        <span class="sm">
          {{
            funcRun?.functionDisplayName ||
            funcRun?.functionName ||
            "Function Run"
          }}
        </span>

        <!-- add a bit more "gap" to the first (dt) element, so the pairs have more space between them -->
        <!-- negative bottom margin to pull it down so all items are visually aligned -->
        <dl
          :class="
            clsx(
              'text-sm flex flex-row items-center gap-2xs [&>dt]:ml-2xs mb-[-0.25em]',
              themeClasses('text-neutral-600', 'text-neutral-400'),
            )
          "
        >
          <template v-if="funcRun?.functionKind">
            <dt><Icon name="func" size="xs" /></dt>
            <dd>{{ funcRun.functionKind }}</dd>
          </template>

          <template v-if="funcRun?.componentName">
            <dt></dt>
            <dd>{{ funcRun.componentName }}</dd>
          </template>

          <template v-if="funcRun?.actionKind">
            <dt><Icon name="play" size="xs" /></dt>
            <dd>{{ funcRun.actionKind }}</dd>
          </template>
        </dl>
        <FuncRunStatusBadge v-if="funcRun && status" :status="status" />
      </div>

      <!-- Action buttons -->
      <div class="flex gap-2">
        <slot name="actions"></slot>
      </div>
    </header>

    <!-- Error banner for hinting -->
    <StatusBox
      v-if="errorHint"
      kind="error"
      class="mx-xs mt-sm mb-xs"
      :text="errorHint"
    />

    <div
      :class="
        clsx(
          'grid grid-cols-[1fr_1fr] grid-rows-[1fr_1fr_1fr] gap-xs p-xs h-full overflow-hidden',
          !errorHint && 'pt-xs',
        )
      "
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
import { computed, ref, watch } from "vue";
import { Icon, themeClasses, IconButton } from "@si/vue-lib/design-system";
import { useRoute, useRouter } from "vue-router";
import clsx from "clsx";
import CodeViewer from "@/components/CodeViewer.vue";
import { FuncRun } from "../api_composables/func_run";
import FuncRunStatusBadge from "../FuncRunStatusBadge.vue";
import GridItemWithLiveHeader from "./GridItemWithLiveHeader.vue";
import StatusBox from "./StatusBox.vue";

const props = defineProps<{
  funcRun: FuncRun;
  status: string;
  logText: string;
  functionCode: string;
  argsJson: string;
  resultJson: string;
  isLive: boolean;
  errorHint?: string;
  errorMessageRaw?: string;
}>();

const logText = computed(() => {
  if (props.errorHint && props.errorMessageRaw)
    return `${props.logText}\n${props.errorHint}\nRaw error message: ${props.errorMessageRaw}`;
  if (props.errorHint && !props.errorMessageRaw)
    return `${props.logText}\n${props.errorHint}`;
  if (!props.errorHint && props.errorMessageRaw)
    return `${props.logText}\nRaw error message: ${props.errorMessageRaw}`;
  return props.logText;
});

const router = useRouter();
const route = useRoute();
const logsContainer = ref<InstanceType<typeof CodeViewer> | null>(null);

// Navigate back to explore_grid view
const navigateBack = () => {
  router.push({
    name: "new-hotness",
    params: {
      workspacePk: route.params.workspacePk,
      changeSetId: route.params.changeSetId,
    },
    query: { retainSessionState: 1 },
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
